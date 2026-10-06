//! For a file's wall records, count what the wall readers find: location
//! lines and arcs (`scan_wall_lines`, `scan_wall_arcs`) and the orientation
//! word (`scan_wall_orientations`) that decides whether a wall is drawn
//! centred on its line.
//!
//! ```bash
//! cargo run --release --example probe_r3_wall_lines -- FILE.rvt...
//! ```

use rvt::RevitFile;
use std::collections::{BTreeMap, BTreeSet};

fn main() -> anyhow::Result<()> {
    for path in std::env::args().skip(1) {
        let mut rf = RevitFile::open(&path)?;
        let version = rf.basic_file_info()?.version;
        let walls: BTreeSet<u32> =
            rvt::partition_element_records_2023::scan_records(&mut rf, version)
                .iter()
                .filter(|r| r.builtin_category == rvt::partition_element_records::OST_WALLS)
                .filter(|r| r.is_placed_instance())
                .map(|r| r.element_id)
                .collect();
        let pcs = rvt::partition_compound_structure::scan_wall_lines(&mut rf, version, &walls)?;
        let arcs = rvt::partition_compound_structure::scan_wall_arcs(&mut rf, version, &walls)?;
        let orientations =
            rvt::partition_compound_structure::scan_wall_orientations(&mut rf, version, &walls)?;
        let mut words: BTreeMap<(u32, u32), usize> = BTreeMap::new();
        for o in orientations.values() {
            *words.entry((o.location_line, o.word)).or_default() += 1;
        }
        println!(
            "{version}: {} walls, {} lines, {} arcs, {} orientations; (location line, word): {:?}",
            walls.len(),
            pcs.len(),
            arcs.len(),
            orientations.len(),
            words
        );
    }
    Ok(())
}
