//! Print every native record of the given ElementIds with its curve: the
//! `GLine` or `GArc` objects of each copy, to see whether a sketch line's
//! copies in several partitions agree.
//!
//! ```bash
//! cargo run --release --example probe_r6_copies -- FILE.rvt ID...
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
    rvt::native_document::extract(&mut rf, &options, |record| {
        println!("== {} {:?}", record.identity.element_id, record.identity);
        if let Some(graph) = record.graph {
            for object in graph
                .objects
                .iter()
                .filter(|o| o.class_name.starts_with('G'))
            {
                println!("   {} {}", object.class_name, object.fields);
            }
        }
        Ok(())
    })?;
    // And the byte reader's curve of each copy.
    let version = rf.basic_file_info()?.version;
    let ids32: BTreeSet<u32> = args[1..].iter().filter_map(|s| s.parse().ok()).collect();
    println!(
        "scan_sketch_curves: {:?}",
        rvt::partition_beam_axes::scan_sketch_curves(&mut rf, version, &ids32)?
    );
    Ok(())
}
