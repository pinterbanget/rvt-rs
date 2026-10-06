//! Print the `InstanceInfo` objects of one element's saved graphics graph
//! (channel 103), as the native reader decodes them, to compare their fields
//! between releases.
//!
//! ```bash
//! cargo run --release --example probe_r3_instance_info -- FILE.rvt ELEMENT_ID
//! ```

use rvt::RevitFile;
use std::collections::BTreeSet;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [path, id] = args.as_slice() else {
        anyhow::bail!("usage: probe_r3_instance_info FILE.rvt ELEMENT_ID");
    };
    let id: u64 = id.parse()?;
    let mut rf = RevitFile::open(path)?;
    let options = rvt::native_document::Options {
        selected_ids: BTreeSet::from([id]),
        channels: BTreeSet::from([103]),
        max_graph_values: 1_000_000,
        max_graph_objects: 1_000_000,
        ..Default::default()
    };
    rvt::native_document::extract(&mut rf, &options, |record| {
        if let Some(graph) = &record.graph {
            for (index, object) in graph.objects.iter().enumerate() {
                if index < 3
                    || object.class_name == "InstanceInfo"
                    || object.class_name == "GInstance"
                {
                    println!(
                        "object {index} {}: {}",
                        object.class_name,
                        serde_json::to_string(&object.fields)?
                    );
                }
            }
        }
        Ok(())
    })?;
    Ok(())
}
