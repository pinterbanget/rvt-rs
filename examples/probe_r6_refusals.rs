//! Count a file's native graph refusals (channel 102) over every declared
//! element: complete and refused graphs, and the refusals by diagnostic.
//!
//! ```bash
//! cargo run --release --example probe_r6_refusals -- FILE.rvt...
//! ```

use rvt::RevitFile;
use std::collections::{BTreeMap, BTreeSet};

fn main() -> anyhow::Result<()> {
    for path in std::env::args().skip(1) {
        let mut rf = RevitFile::open(&path)?;
        let declared: BTreeSet<u64> =
            rvt::elem_table::declared_ids(&rvt::elem_table::parse_records(&mut rf)?)
                .into_iter()
                .map(u64::from)
                .collect();
        let options = rvt::native_document::Options {
            selected_ids: declared,
            channels: BTreeSet::from([102]),
            max_graph_values: 1_000_000,
            max_graph_objects: 1_000_000,
            ..Default::default()
        };
        let mut diagnostics: BTreeMap<String, usize> = BTreeMap::new();
        let summary = rvt::native_document::extract(&mut rf, &options, |record| {
            if let Some(d) = record.diagnostic {
                let key: String = d
                    .split(": ")
                    .last()
                    .unwrap_or_default()
                    .chars()
                    .take(70)
                    .collect();
                *diagnostics.entry(key).or_default() += 1;
            }
            Ok(())
        })?;
        println!(
            "{path}: complete {} refused {}",
            summary.complete_graph_records, summary.unsupported_graph_records
        );
        let mut top: Vec<_> = diagnostics.into_iter().collect();
        top.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        for (d, n) in top.into_iter().take(6) {
            println!("   {n:6} {d}");
        }
    }
    Ok(())
}
