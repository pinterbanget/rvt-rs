//! Summarise what the native saved-graphics reader gets from a file: the
//! graph records it decodes or refuses (by class), and for the elements a
//! 32-bit export draws as boxes, how many come back with mesh primitives and
//! which diagnostics the others carry.
//!
//! ```bash
//! cargo run --release --example probe_r3_saved_scene -- FILE.rvt
//! ```

use rvt::RevitFile;
use std::collections::{BTreeMap, BTreeSet};

fn main() -> anyhow::Result<()> {
    for path in std::env::args().skip(1) {
        let mut rf = RevitFile::open(&path)?;
        let version = rf.basic_file_info()?.version;
        // The ids a 32-bit export draws as record boxes: every placed record.
        let ids: BTreeSet<u64> = if rvt::partition_element_records_2023::reads_release(version) {
            rvt::partition_element_records_2023::scan_records(&mut rf, version)
                .iter()
                .filter(|r| r.is_placed_instance())
                .map(|r| u64::from(r.element_id))
                .collect()
        } else {
            BTreeSet::new()
        };
        let options = rvt::native_document::Options {
            selected_ids: ids.clone(),
            max_graph_values: 1_000_000,
            max_graph_objects: 1_000_000,
            ..Default::default()
        };
        let scene = match rvt::native_saved_scene::extract_at_detail(&mut rf, &options, 3) {
            Ok(scene) => scene,
            Err(error) => {
                println!("{version} {path}: extraction failed: {error:#}");
                continue;
            }
        };
        println!("{version} {path}: {} selected ids", ids.len());
        for (index, s) in scene.summaries.iter().enumerate() {
            println!(
                "  pass {index}: indexed {} selected {} emitted {} complete graphs {} unsupported {} refused {:?}",
                s.indexed_elements,
                s.selected_indexed_elements,
                s.emitted_records,
                s.complete_graph_records,
                s.unsupported_graph_records,
                s.refused_graph_classes.iter().take(8).collect::<Vec<_>>()
            );
        }
        let with_mesh = scene
            .elements
            .iter()
            .filter(|e| !e.meshes.primitives.is_empty())
            .count();
        let mut diagnostics: BTreeMap<String, usize> = BTreeMap::new();
        let mut examples: BTreeMap<String, Vec<u64>> = BTreeMap::new();
        for element in scene
            .elements
            .iter()
            .filter(|e| e.meshes.primitives.is_empty())
        {
            for d in &element.meshes.diagnostics {
                let key: String = d.chars().take(90).collect();
                *diagnostics.entry(key.clone()).or_default() += 1;
                let ids = examples.entry(key).or_default();
                if ids.len() < 4 {
                    ids.push(element.id);
                }
            }
        }
        println!(
            "  elements {} with mesh {} without {}",
            scene.elements.len(),
            with_mesh,
            scene.elements.len() - with_mesh
        );
        let mut top: Vec<_> = diagnostics.into_iter().collect();
        top.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        for (d, n) in top.into_iter().take(10) {
            println!(
                "    {n:5}  {d}  e.g. {:?}",
                examples.get(&d).cloned().unwrap_or_default()
            );
        }
    }
    Ok(())
}
