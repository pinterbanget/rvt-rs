//! Print the native graph status and diagnostic of the given ElementIds
//! (channel 102).
//!
//! ```bash
//! cargo run --release --example probe_r6_graph_status -- FILE.rvt ID...
//! ```

use rvt::RevitFile;
use std::collections::BTreeSet;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut rf = RevitFile::open(&args[0])?;
    let ids: BTreeSet<u64> = args[1..].iter().filter_map(|s| s.parse().ok()).collect();
    let options = rvt::native_document::Options {
        selected_ids: ids,
        channels: BTreeSet::from([102]),
        max_graph_values: 1_000_000,
        max_graph_objects: 1_000_000,
        ..Default::default()
    };
    let summary = rvt::native_document::extract(&mut rf, &options, |record| {
        println!(
            "{} {:?} {} {:?} body {}",
            record.identity.element_id,
            record.class_name,
            record.status,
            record.diagnostic,
            record.source.body_bytes
        );
        Ok(())
    })?;
    println!("refused classes {:?}", summary.refused_graph_classes);
    Ok(())
}
