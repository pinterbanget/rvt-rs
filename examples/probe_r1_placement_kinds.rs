//! Count the placement-kind words (`+0x42`) of every element record a
//! 32-bit release's scanner finds, by category sign: Revit 2016 to 2018
//! write a placed instance as `0xffffef6f` where 2019 and later write
//! `0xffffef7f`.
//!
//! ```bash
//! cargo run --release --example probe_r1_placement_kinds -- FILE.rvt...
//! ```

use rvt::RevitFile;
use std::collections::BTreeMap;

fn main() -> anyhow::Result<()> {
    for path in std::env::args().skip(1) {
        let mut rf = RevitFile::open(&path)?;
        let version = rf.basic_file_info()?.version;
        let records = rvt::partition_element_records_2023::scan_records(&mut rf, version);
        let mut kinds: BTreeMap<u32, usize> = BTreeMap::new();
        for record in &records {
            *kinds.entry(record.placement_kind).or_default() += 1;
        }
        let summary: Vec<String> = kinds
            .iter()
            .map(|(kind, count)| format!("{kind:#010x}:{count}"))
            .collect();
        println!(
            "{version}\t{} records\t{}",
            records.len(),
            summary.join(" ")
        );
    }
    Ok(())
}
