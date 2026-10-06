//! For each type in IDS_FILE: whether the BuiltInParameter centroid test
//! (`type_i_sections`) reads it as an I section, whether its own solid is
//! doubly symmetric (`native_section_doubly_symmetric`), and the section
//! `native_i_sections` reads from its family parameters.
//!
//! ```bash
//! cargo run --release --example probe_r5_section_symmetry -- IDS_FILE FILE.rvt
//! ```

use rvt::RevitFile;
use std::collections::BTreeSet;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [ids, path] = args.as_slice() else {
        anyhow::bail!("usage: IDS_FILE FILE.rvt");
    };
    let types: BTreeSet<u32> = std::fs::read_to_string(ids)?
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect();
    let mut rf = RevitFile::open(path)?;
    let version = rf.basic_file_info()?.version;
    let builtin = rvt::partition_type_parameters::type_i_sections(&mut rf, version, &types);
    let native = rvt::partition_type_parameters::native_i_sections(&mut rf, &types);
    let options = rvt::native_document::Options {
        selected_ids: types.iter().map(|&t| u64::from(t)).collect(),
        channels: BTreeSet::from([102]),
        max_graph_values: 1_000_000,
        max_graph_objects: 1_000_000,
        ..Default::default()
    };
    let mut rows = Vec::new();
    rvt::native_document::extract(&mut rf, &options, |record| {
        if let Some(graph) = &record.graph {
            let id = u32::try_from(record.identity.element_id).unwrap_or(0);
            rows.push((
                id,
                rvt::partition_type_parameters::native_section_doubly_symmetric(graph),
            ));
        }
        Ok(())
    })?;
    let (mut agree, mut total) = (0, 0);
    for (id, symmetric) in rows {
        let is_i = builtin.contains_key(&id);
        total += 1;
        agree += usize::from(Some(is_i) == symmetric || (!is_i && symmetric.is_none()));
        println!(
            "{version} type {id}: centroid-I {is_i} symmetric {symmetric:?} native {:?}",
            native
                .get(&id)
                .map(|s| (s.width_feet, s.depth_feet, s.web_feet, s.flange_feet))
        );
    }
    println!("{version}: centroid test and symmetry agree on {agree} of {total} types");
    Ok(())
}
