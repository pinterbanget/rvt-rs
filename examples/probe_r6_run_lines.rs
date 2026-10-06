//! Compare a 2024 stair run's lines as RE-52 reads them from its bytes
//! (`partition_stairs::scan_run_lines`) with its native `GLine`s, and run
//! each through `run_sketch`.
//!
//! ```bash
//! cargo run --release --example probe_r6_run_lines -- FILE2024.rvt RUN...
//! ```

use rvt::RevitFile;
use std::collections::BTreeSet;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut rf = RevitFile::open(&args[0])?;
    let version = rf.basic_file_info()?.version;
    let runs: BTreeSet<u32> = args[1..].iter().filter_map(|s| s.parse().ok()).collect();
    let lines = rvt::partition_stairs::scan_run_lines(&mut rf, version, &runs)?;
    let sketches: std::collections::BTreeMap<u32, rvt::partition_stairs::RunSketch> = lines
        .iter()
        .filter_map(|(&id, l)| Some((id, rvt::partition_stairs::run_sketch(l)?)))
        .collect();
    let ends = rvt::partition_stairs::scan_run_ends(&mut rf, version, &sketches)?;
    println!("ends {ends:?}");
    for run in &runs {
        let bytes = lines.get(run);
        println!(
            "{run}: byte lines {:?}, sketch risers {:?}",
            bytes.map(|l| l.len()),
            bytes
                .and_then(|l| rvt::partition_stairs::run_sketch(l))
                .map(|s| s.risers.len())
        );
        if let Some(l) = bytes {
            for line in l.iter().take(4) {
                println!("   {:?} -> {:?}", line.start(), line.end());
            }
        }
    }
    Ok(())
}
