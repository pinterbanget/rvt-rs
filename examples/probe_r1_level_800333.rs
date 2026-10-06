//! Print every `OST_Levels` element record of a file with its container,
//! placement kind and box, from the release's own record reader: the 2024
//! reader on 2024 and later, the 32-bit one on 2016 to 2023.
//!
//! ```bash
//! cargo run --release --example probe_r1_level_800333 -- FILE.rvt...
//! ```

use rvt::RevitFile;

fn main() -> anyhow::Result<()> {
    for path in std::env::args().skip(1) {
        let mut rf = RevitFile::open(&path)?;
        let version = rf.basic_file_info()?.version;
        let declared = rvt::elem_table::declared_ids(&rvt::elem_table::parse_records(&mut rf)?);
        let records = if rvt::partition_element_records_2023::reads_release(version) {
            rvt::partition_element_records_2023::scan_records(&mut rf, version)
                .into_iter()
                .filter(|r| r.builtin_category == rvt::partition_level_records::OST_LEVELS)
                .collect()
        } else {
            rvt::partition_element_records::scan_category_records(
                &mut rf,
                version,
                rvt::partition_level_records::OST_LEVELS,
                &declared,
            )?
        };
        println!(
            "{version} {path}: {} OST_Levels element records",
            records.len()
        );
        if let (Some(marker), Some(bbox_marker)) = (
            rvt::partition_level_records::record_marker(version),
            rvt::partition_element_records::bbox_marker(version),
        ) {
            for stream in rf.partition_stream_names() {
                let Ok(inflated) = rf.inflated_partition(&stream) else {
                    continue;
                };
                let buf = inflated.bytes();
                let chain =
                    rvt::partition_element_records::partition_record_chain(buf, &bbox_marker);
                for r in rvt::partition_level_records::find_level_records_with(
                    &stream, buf, &declared, &marker, &chain,
                ) {
                    println!(
                        "  level record {:>8} {stream} level_element={} container={:#x} placement={:#x}",
                        r.element_id,
                        r.is_level_element(),
                        r.container,
                        r.placement_kind
                    );
                }
            }
        }
        for r in records {
            println!(
                "  {:>8} container={:#x} placement={:#x} box={:?}",
                r.element_id, r.container, r.placement_kind, r.bbox_feet
            );
        }
    }
    Ok(())
}
