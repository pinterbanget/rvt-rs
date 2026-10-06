//! Print the decoded fields of the given ElementIds after the partition
//! schema MVP has run.
//!
//! ```bash
//! cargo run --release --example probe_r6_element_fields -- FILE.rvt ID...
//! ```

use rvt::RevitFile;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut rf = RevitFile::open(&args[0])?;
    let ids: Vec<u32> = args[1..].iter().filter_map(|s| s.parse().ok()).collect();
    let version = rf.basic_file_info()?.version;
    let out = rvt::partition_schema_mvp::recover_partition_schema_mvp(
        &mut rf,
        version,
        Default::default(),
    )?;
    for element in out
        .walls
        .iter()
        .chain(&out.slabs)
        .chain(&out.products)
        .chain(&out.columns)
        .filter(|e| e.id.is_some_and(|id| ids.contains(&id)))
    {
        println!("{:?} {}", element.id, element.class);
        for (name, value) in &element.fields {
            let text = format!("{value:?}");
            println!("   {name}: {}", text.chars().take(160).collect::<String>());
        }
    }
    Ok(())
}
