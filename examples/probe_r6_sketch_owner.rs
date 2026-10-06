//! Find a 32-bit sketch line's owner: the last slot of the second counted
//! list after its record box, as on 2024 (RE-25). Joins a 2016 to 2023
//! file's `OST_SketchLines` records with its 2024 copy's on ElementId and
//! counts where the 32-bit second list's last slot agrees with 2024's
//! `owner_reference`.
//!
//! ```bash
//! cargo run --release --example probe_r6_sketch_owner -- OLD.rvt 2024.rvt
//! ```

use rvt::RevitFile;
use rvt::partition_element_records as per;
use std::collections::BTreeMap;

fn u32_at(buf: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(buf.get(at..at + 4)?.try_into().ok()?))
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (old, new) = (&args[0], &args[1]);
    let mut rf_new = RevitFile::open(new)?;
    let v_new = rf_new.basic_file_info()?.version;
    let declared = rvt::elem_table::declared_ids(&rvt::elem_table::parse_records(&mut rf_new)?);
    let reference: BTreeMap<u32, Option<u32>> =
        per::scan_category_records_multi(&mut rf_new, v_new, &[per::OST_SKETCH_LINES], &declared)?
            .into_iter()
            .map(|r| (r.element_id, r.owner_reference))
            .collect();
    let mut rf = RevitFile::open(old)?;
    let version = rf.basic_file_info()?.version;
    let records: Vec<_> = rvt::partition_element_records_2023::scan_records(&mut rf, version)
        .into_iter()
        .filter(|r| r.builtin_category == per::OST_SKETCH_LINES)
        .collect();
    let (mut shared, mut agree, mut differ, mut none) = (0, 0, 0, 0);
    let mut shown = 0;
    let mut streams: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for record in &records {
        let Some(expected) = reference.get(&record.element_id) else {
            continue;
        };
        shared += 1;
        let buf = streams.entry(record.stream.clone()).or_insert_with(|| {
            rf.inflated_partition(&record.stream)
                .map(|b| b.bytes().to_vec())
                .unwrap_or_default()
        });
        let first_at = record.offset + 52 + 8 + 48;
        let n1 = u32_at(buf, first_at).unwrap_or(u32::MAX) as usize;
        let second_at = first_at + 4 + 4 * n1;
        let n2 = u32_at(buf, second_at).unwrap_or(0) as usize;
        let last = (n2 > 0 && n2 < 1024)
            .then(|| u32_at(buf, second_at + 4 * n2))
            .flatten();
        match (last, expected) {
            (Some(a), Some(b)) if a == *b => agree += 1,
            (_, None) => none += 1,
            _ => {
                differ += 1;
                if shown < 6 {
                    shown += 1;
                    let from = first_at;
                    let to = (first_at + 96).min(buf.len());
                    println!(
                        "  {} expected {:?} n1 {} n2 {} last {:?}\n    {}",
                        record.element_id,
                        expected,
                        n1,
                        n2,
                        last,
                        buf[from..to]
                            .chunks(4)
                            .map(|c| c.iter().map(|b| format!("{b:02x}")).collect::<String>())
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                }
            }
        }
    }
    println!(
        "{version}: {} sketch-line records, {shared} shared with 2024; second list's last slot agrees {agree}, differs {differ}, 2024 has none {none}",
        records.len()
    );
    Ok(())
}
