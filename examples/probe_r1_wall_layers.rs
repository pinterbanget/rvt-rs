//! For one wall and its type, print what each wall reader returns: the
//! type's compound layers, the wall's orientation, and the materials the
//! layers may name. Used to find which reader stops a 32-bit release's wall
//! from taking its layer set.
//!
//! ```bash
//! cargo run --release --example probe_r1_wall_layers -- FILE.rvt WALL_ID TYPE_ID
//! ```

use rvt::RevitFile;
use std::collections::BTreeSet;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [path, wall, wall_type] = args.as_slice() else {
        anyhow::bail!("usage: probe_r1_wall_layers FILE.rvt WALL_ID TYPE_ID");
    };
    let (wall, wall_type): (u32, u32) = (wall.parse()?, wall_type.parse()?);
    let mut rf = RevitFile::open(path)?;
    let version = rf.basic_file_info()?.version;
    let declared = rvt::elem_table::declared_ids(&rvt::elem_table::parse_records(&mut rf)?);
    let (materials, names) = rvt::partition_materials::scan_materials_2023(&mut rf, &declared);
    println!(
        "release {version}: {} materials, {} named; wall declared {}, type declared {}",
        materials.len(),
        names.len(),
        declared.contains(&wall),
        declared.contains(&wall_type)
    );
    let types = BTreeSet::from([wall_type]);
    let layers = rvt::partition_compound_structure::scan_type_layers(
        &mut rf, version, &types, &materials, &declared,
    )?;
    match layers.get(&wall_type) {
        Some(layers) => {
            for layer in layers {
                let name = layer
                    .material
                    .and_then(|id| names.get(&id).cloned())
                    .unwrap_or_default();
                println!(
                    "  layer {:.4} ft  material {:?} {name}",
                    layer.width_feet, layer.material
                );
            }
        }
        None => println!("  no layers for type {wall_type}"),
    }
    let orientations = rvt::partition_compound_structure::scan_wall_orientations(
        &mut rf,
        version,
        &BTreeSet::from([wall]),
    )?;
    println!("  orientation {:?}", orientations.get(&wall));
    Ok(())
}
