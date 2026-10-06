//! Count a 32-bit file's element records framed without an outline: the
//! marker's `Outline` tag is `00 00` and no box follows. Counted by
//! category, with how many of their ids the outlined scan also finds.
//!
//! ```bash
//! cargo run --release --example probe_r6_no_outline -- FILE.rvt
//! ```

use rvt::RevitFile;
use std::collections::{BTreeMap, BTreeSet};

fn main() -> anyhow::Result<()> {
    for path in std::env::args().skip(1) {
        let mut rf = RevitFile::open(&path)?;
        let version = rf.basic_file_info()?.version;
        let declared = rvt::elem_table::declared_ids(&rvt::elem_table::parse_records(&mut rf)?);
        let (marker, header) = rvt::partition_element_records_2023::record_marker(&mut rf).unwrap();
        let outlined: BTreeSet<u32> =
            rvt::partition_element_records_2023::scan_records(&mut rf, version)
                .iter()
                .map(|r| r.element_id)
                .collect();
        let mut bare = marker;
        bare[0] = 0;
        bare[1] = 0;
        let mut by_category: BTreeMap<i64, (BTreeSet<u32>, usize)> = BTreeMap::new();
        let mut backs: BTreeMap<usize, usize> = BTreeMap::new();
        for stream in rf.partition_stream_names() {
            let Ok(inflated) = rf.inflated_partition(&stream) else {
                continue;
            };
            let buf = inflated.bytes();
            for at in memchr::memmem::find_iter(buf, &bare) {
                let Some(id_at) = [50usize, 52].iter().find_map(|back| {
                    let id_at = at.checked_sub(*back)?;
                    let id = u32::from_le_bytes(buf[id_at..id_at + 4].try_into().ok()?);
                    let tag = u16::from_le_bytes(buf[id_at + 8..id_at + 10].try_into().ok()?);
                    (declared.contains(&id) && tag == header).then_some(id_at)
                }) else {
                    continue;
                };
                let id = u32::from_le_bytes(buf[id_at..id_at + 4].try_into()?);
                *backs.entry(at - id_at).or_default() += 1;
                let category = i64::from_le_bytes(buf[id_at + 14..id_at + 22].try_into()?);
                let entry = by_category.entry(category).or_default();
                entry.0.insert(id);
                entry.1 += 1;
            }
        }
        println!("{version} {path}  id before marker {backs:?}");
        for (category, (ids, frames)) in by_category {
            let also = ids.iter().filter(|id| outlined.contains(id)).count();
            println!(
                "  {category:9}  {:5} ids ({frames} frames), {also} also outlined",
                ids.len()
            );
        }
    }
    Ok(())
}
