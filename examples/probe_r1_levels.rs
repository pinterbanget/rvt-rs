//! Print the Levels `partition_level_records::recover_partition_levels`
//! reads from each file: ElementId, name, elevation and Building Story.
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
        for level in levels {
            println!(
                "  {:>8}  {:>9.3} ft  story={:?}  {:?}",
                level.element_id, level.elevation_feet, level.building_story, level.name
            );
        }
    }
    Ok(())
}
