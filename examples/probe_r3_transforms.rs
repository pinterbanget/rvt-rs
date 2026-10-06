//! How many of the given ElementIds `scan_instance_transforms` reads a
//! transform for, and the first few, on each file.
//!
//! ```bash
//! cargo run --release --example probe_r3_transforms -- IDS_FILE FILE.rvt...
//! ```
//! `IDS_FILE` holds one ElementId per line.

use rvt::RevitFile;
use std::collections::BTreeSet;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let ids_file = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("usage: IDS_FILE FILE.rvt..."))?;
    let ids: BTreeSet<u32> = std::fs::read_to_string(ids_file)?
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect();
    for path in args {
        let mut rf = RevitFile::open(&path)?;
        let version = rf.basic_file_info()?.version;
        let found =
            rvt::partition_instance_transforms::scan_instance_transforms(&mut rf, version, &ids)?;
        println!(
            "{version}: {} of {} ids have a transform",
            found.len(),
            ids.len()
        );
        for (id, t) in found.iter().take(3) {
            println!("  {id}: {t:?}");
        }
    }
    Ok(())
}
