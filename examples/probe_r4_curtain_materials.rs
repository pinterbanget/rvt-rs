//! The material ElementId and name of each curtain mullion and panel type,
//! from `partition_curtain_materials` (on the releases it reads).
//!
//! ```bash
//! cargo run --release --example probe_r4_curtain_materials -- FILE.rvt
//! ```

use rvt::RevitFile;

fn main() -> anyhow::Result<()> {
    for path in std::env::args().skip(1) {
        let mut rf = RevitFile::open(&path)?;
        let version = rf.basic_file_info()?.version;
        let declared = rvt::elem_table::declared_ids(&rvt::elem_table::parse_records(&mut rf)?);
        let names = rvt::partition_materials::scan_material_names(&mut rf, version, &declared)?;
        let materials = names.keys().copied().collect();
        let types = rvt::partition_curtain_materials::scan_curtain_type_materials(
            &mut rf, version, &materials,
        )?;
        println!("{version}: {} curtain types with a material", types.len());
        for (t, m) in types.iter().take(8) {
            println!("  type {t} material {m} {:?}", names.get(m));
        }
    }
    Ok(())
}
