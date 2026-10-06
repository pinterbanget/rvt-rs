//! Print the extent of each given element's saved mesh (feet), and, with
//! `X0 X1` given in the environment as `SLAB=x0,x1`, how many of its triangles
//! have their centroid with x in that range, to test whether a saved solid
//! fills a strip.
//!
//! ```bash
//! cargo run --release --example probe_r6_mesh_extent -- FILE.rvt ID...
//! ```

use rvt::RevitFile;
use std::collections::BTreeSet;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut rf = RevitFile::open(&args[0])?;
    let ids: BTreeSet<u64> = args[1..].iter().filter_map(|s| s.parse().ok()).collect();
    let options = rvt::native_document::Options {
        selected_ids: ids,
        max_graph_values: 1_000_000,
        max_graph_objects: 1_000_000,
        ..Default::default()
    };
    let strip: Option<(f64, f64)> = std::env::var("SLAB").ok().and_then(|s| {
        let (a, b) = s.split_once(',')?;
        Some((a.parse().ok()?, b.parse().ok()?))
    });
    let scene = rvt::native_saved_scene::extract_at_detail(&mut rf, &options, 3)?;
    for element in &scene.elements {
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        let mut in_strip = 0usize;
        let mut total = 0usize;
        for p in &element.meshes.primitives {
            for v in &p.vertices {
                for a in 0..3 {
                    lo[a] = lo[a].min(v[a]);
                    hi[a] = hi[a].max(v[a]);
                }
            }
            for t in &p.triangles {
                total += 1;
                let cx = t.iter().map(|&i| p.vertices[i as usize][0]).sum::<f64>() / 3.0;
                if strip.is_some_and(|(a, b)| cx > a && cx < b) {
                    in_strip += 1;
                }
            }
        }
        println!(
            "{} primitives {} extent {:?} .. {:?} triangles {} in strip {}",
            element.id,
            element.meshes.primitives.len(),
            lo,
            hi,
            total,
            in_strip
        );
    }
    Ok(())
}
