//! Print the native graph objects (class and a few fields) of the given
//! ElementIds on one channel, to see what the schema-driven reader decodes
//! for a type or a family on a release.
//!
//! ```bash
//! cargo run --release --example probe_r4_native_objects -- FILE.rvt CHANNEL ID...
//! ```

use rvt::RevitFile;
use std::collections::BTreeSet;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [path, channel, ids @ ..] = args.as_slice() else {
        anyhow::bail!("usage: FILE.rvt CHANNEL ID...");
    };
    let ids: BTreeSet<u64> = ids.iter().filter_map(|i| i.parse().ok()).collect();
    let mut rf = RevitFile::open(path)?;
    let options = rvt::native_document::Options {
        selected_ids: ids,
        channels: BTreeSet::from([channel.parse()?]),
        max_graph_values: 1_000_000,
        max_graph_objects: 1_000_000,
        ..Default::default()
    };
    rvt::native_document::extract(&mut rf, &options, |record| {
        println!(
            "== element {} graph {}",
            record.identity.element_id,
            record.graph.is_some()
        );
        if let Some(graph) = &record.graph {
            if let Ok(needle) = std::env::var("FIND") {
                for (index, object) in graph.objects.iter().enumerate() {
                    let text = serde_json::to_string(&object.fields)?;
                    if let Some(at) = text.find(&needle) {
                        let from = at.saturating_sub(160);
                        println!(
                            "  {index} {}: ...{}",
                            object.class_name,
                            &text[from..(at + needle.len() + 40).min(text.len())]
                        );
                    }
                }
                return Ok(());
            }
            for (index, object) in graph.objects.iter().enumerate().take(12) {
                let text = serde_json::to_string(&object.fields)?;
                let limit = if std::env::var_os("FULL").is_some() && index == 0 {
                    20_000
                } else {
                    260
                };
                let short: String = text.chars().take(limit).collect();
                println!("  {index} {}: {short}", object.class_name);
            }
        }
        Ok(())
    })?;
    Ok(())
}
