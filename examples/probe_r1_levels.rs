//! Print the Levels `partition_level_records::recover_partition_levels`
//! reads from each file: ElementId, name, elevation and Building Story.
//!
//! On a 32-bit release (2016 to 2023) it also lists every `OST_Levels`
//! record `find_level_records_2023` finds before the storey-set check, so a
//! file whose Levels are dropped by that check shows which record failed.
//!
//! ```bash
//! cargo run --release --example probe_r1_levels -- FILE.rvt...
//! ```

use rvt::RevitFile;

fn main() -> anyhow::Result<()> {
    for path in std::env::args().skip(1) {
        let mut rf = RevitFile::open(&path)?;
        let version = rf.basic_file_info()?.version;
        let levels = rvt::partition_level_records::recover_partition_levels(&mut rf, version)?;
        println!("{version} {path}: {} levels", levels.len());
        if rvt::partition_element_records_2023::reads_release(version) {
            let declared = rvt::elem_table::declared_ids(&rvt::elem_table::parse_records(&mut rf)?);
            if let Some((full, header_tag)) =
                rvt::partition_element_records_2023::record_marker(&mut rf)
            {
                let mut marker = [0u8; 6];
                marker.copy_from_slice(&full[2..]);
                for stream in rf.partition_stream_names() {
                    let Ok(inflated) = rf.inflated_partition(&stream) else {
                        continue;
                    };
                    for record in rvt::partition_level_records::find_level_records_2023(
                        &stream,
                        inflated.bytes(),
                        &declared,
                        header_tag,
                        &marker,
                    ) {
                        println!(
                            "  record {:>8} {stream} level_element={} container={:#x} placement={:#x}",
                            record.element_id,
                            record.is_level_element(),
                            record.container,
                            record.placement_kind
                        );
                    }
                }
            }
        }
        for level in levels {
            println!(
                "  {:>8}  {:>9.3} ft  story={:?}  {:?}",
                level.element_id, level.elevation_feet, level.building_story, level.name
            );
        }
    }
    Ok(())
}
