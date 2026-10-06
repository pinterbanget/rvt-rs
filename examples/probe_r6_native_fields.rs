//! Print the scalar and ElementId fields of the native graph objects of the
//! given ElementIds (channel 102), one per line, so two releases' objects can
//! be compared. `CLASS=name` keeps only objects of that class.
//!
//! ```bash
//! cargo run --release --example probe_r6_native_fields -- FILE.rvt ID...
//! ```

use rvt::RevitFile;
use std::collections::BTreeSet;

fn scalar(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Number(_)
        | serde_json::Value::Bool(_)
        | serde_json::Value::String(_) => Some(value.to_string()),
        serde_json::Value::Object(map) => {
            let mut inner = map.get("m_id").or_else(|| map.get("m_id64"))?;
            loop {
                match inner {
                    serde_json::Value::Number(n) => return Some(format!("id {n}")),
                    serde_json::Value::Object(m) => {
                        inner = m.get("m_id").or_else(|| m.get("m_id64"))?
                    }
                    _ => return None,
                }
            }
        }
        serde_json::Value::Array(items)
            if items.len() <= 4 && items.iter().all(|v| v.is_number()) =>
        {
            Some(value.to_string())
        }
        _ => None,
    }
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut rf = RevitFile::open(&args[0])?;
    let ids: BTreeSet<u64> = args[1..].iter().filter_map(|s| s.parse().ok()).collect();
    let only = std::env::var("CLASS").ok();
    let options = rvt::native_document::Options {
        selected_ids: ids,
        channels: BTreeSet::from([102]),
        max_graph_values: 1_000_000,
        max_graph_objects: 1_000_000,
        ..Default::default()
    };
    rvt::native_document::extract(&mut rf, &options, |record| {
        let Some(graph) = record.graph else {
            return Ok(());
        };
        println!("== {}", record.identity.element_id);
        for (index, object) in graph.objects.iter().enumerate() {
            if only.as_ref().is_some_and(|c| c != &object.class_name) {
                continue;
            }
            let Some(fields) = object.fields.as_object() else {
                continue;
            };
            for (name, value) in fields {
                if let Some(text) = scalar(value) {
                    println!("  {index:3} {:24} {name:32} {text}", object.class_name);
                }
            }
        }
        Ok(())
    })?;
    Ok(())
}
