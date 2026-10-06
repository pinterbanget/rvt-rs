//! Compare each stair run type 2024's byte reader reads (RE-52,
//! `partition_stairs::scan_run_types`) with its native `StairsRunType`
//! object on the same file.
//!
//! ```bash
//! cargo run --release --example probe_r6_run_types -- FILE2024.rvt
//! ```

use rvt::RevitFile;
use std::collections::BTreeSet;

fn main() -> anyhow::Result<()> {
    for path in std::env::args().skip(1) {
        let mut rf = RevitFile::open(&path)?;
        let version = rf.basic_file_info()?.version;
        let declared = rvt::elem_table::declared_ids(&rvt::elem_table::parse_records(&mut rf)?);
        let records = rvt::partition_type_records::scan_type_records(
            &mut rf,
            version,
            rvt::partition_element_records::OST_STAIRS_RUNS,
            &declared,
        )?;
        let ids: BTreeSet<u32> = rvt::partition_type_records::type_definition_ids(&records)
            .into_iter()
            .collect();
        let bytes = rvt::partition_stairs::scan_run_types(&mut rf, version, &ids)?;
        let options = rvt::native_document::Options {
            selected_ids: ids.iter().map(|&id| u64::from(id)).collect(),
            channels: BTreeSet::from([102]),
            max_graph_values: 1_000_000,
            max_graph_objects: 1_000_000,
            ..Default::default()
        };
        println!("{path}");
        rvt::native_document::extract(&mut rf, &options, |record| {
            let Some(graph) = record.graph else {
                return Ok(());
            };
            let root = &graph.objects[0];
            let f = |name: &str| root.fields.get(name).cloned().unwrap_or_default();
            let id = record.identity.element_id as u32;
            println!(
                "  {id} {}: bytes {:?}\n     native depth {} tread {} riser {} nosing {} mono {} hasTread {} hasRiser {} slanted {} connection {}",
                root.class_name,
                bytes.get(&id),
                f("m_structuralDepth"),
                f("m_treadThickness"),
                f("m_riserThickness"),
                f("m_nosingLength"),
                f("m_hasMonolithicSupport"),
                f("m_hasTread"),
                f("m_hasRiser"),
                f("m_riserIsSlanted"),
                f("m_riserTreadConnection"),
            );
            Ok(())
        })?;
    }
    Ok(())
}
