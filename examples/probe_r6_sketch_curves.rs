//! Compare the sketch lines of each sketched element on a 2016 to 2023 file
//! with its 2024 copy: whether the box solve closes (RE-25), and how many of
//! its lines read as a line or an arc on each release (RE-50, RE-96).
//!
//! ```bash
//! cargo run --release --example probe_r6_sketch_curves -- OLD.rvt 2024.rvt [OWNER]
//! ```

use rvt::RevitFile;
use rvt::element_record_plan_profiles as erpp;
use rvt::partition_beam_axes as pba;
use rvt::partition_element_records as per;
use std::collections::{BTreeMap, BTreeSet};

type Lines = Vec<per::PartitionElementRecord>;

fn lines_of(path: &str) -> anyhow::Result<(RevitFile, u32, Lines)> {
    let mut rf = RevitFile::open(path)?;
    let version = rf.basic_file_info()?.version;
    let lines: Lines = if rvt::partition_element_records_2023::reads_release(version) {
        let lines: Lines = rvt::partition_element_records_2023::scan_records(&mut rf, version)
            .into_iter()
            .filter(|r| r.builtin_category == per::OST_SKETCH_LINES)
            .collect();
        let bare: Vec<_> =
            rvt::partition_element_records_2023::scan_unoutlined_records(&mut rf, version)
                .into_iter()
                .filter(|r| r.builtin_category == per::OST_SKETCH_LINES)
                .collect();
        let ids = lines
            .iter()
            .map(|r| r.element_id)
            .chain(bare.iter().map(|r| r.element_id))
            .collect();
        let sketches = rvt::partition_schema_mvp::native_sketches(&mut rf, &ids);
        rvt::partition_schema_mvp::sketch_lines_32bit(lines, &bare, &sketches.curves)
    } else {
        let declared = rvt::elem_table::declared_ids(&rvt::elem_table::parse_records(&mut rf)?);
        per::scan_category_records_multi(&mut rf, version, &[per::OST_SKETCH_LINES], &declared)?
    };
    Ok((rf, version, lines))
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let only: Option<u32> = args.get(2).and_then(|s| s.parse().ok());
    let mut sides = Vec::new();
    for path in &args[..2] {
        let (mut rf, version, lines) = lines_of(path)?;
        let closed = erpp::plan_profiles_from_sketch_line_records(&lines);
        let ids: BTreeSet<u32> = lines.iter().map(|r| r.element_id).collect();
        let bounded = pba::scan_bounded_lines(&mut rf, version, &ids)?;
        let curves = pba::scan_sketch_curves(&mut rf, version, &ids)?;
        let lists = erpp::scan_sketch_curve_lists(&mut rf, version, &ids);
        let mut by_owner: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
        let mut boxes = BTreeMap::new();
        for r in &lines {
            if let Some(owner) = r.owner_reference {
                by_owner.entry(owner).or_default().insert(r.element_id);
            }
            boxes.entry(r.element_id).or_insert(r.bbox_feet);
        }
        sides.push((version, closed, bounded, curves, lists, by_owner, boxes));
    }
    let (old, new) = (&sides[0], &sides[1]);
    println!("owner   lines old/new  closed old/new  line old/new  arc old/new  list old/new");
    for (owner, ids_new) in &new.5 {
        if only.is_some_and(|o| o != *owner) {
            continue;
        }
        let ids_old = old.5.get(owner).cloned().unwrap_or_default();
        let count = |side: &(
            u32,
            _,
            BTreeMap<u32, pba::BoundedLine>,
            BTreeMap<u32, pba::SketchCurve>,
            _,
            _,
            _,
        ),
                     ids: &BTreeSet<u32>| {
            (
                ids.iter().filter(|id| side.2.contains_key(id)).count(),
                ids.iter()
                    .filter(|id| side.3.get(id).is_some_and(|c| c.arc.is_some()))
                    .count(),
            )
        };
        let (lo, ao) = count(old, &ids_old);
        let (ln, an) = count(new, ids_new);
        println!(
            "{owner:7} {:5}/{:<5}    {:5}/{:<5}     {:4}/{:<4}    {:4}/{:<4}   {:?}/{:?}",
            ids_old.len(),
            ids_new.len(),
            old.1.contains_key(owner),
            new.1.contains_key(owner),
            lo,
            ln,
            ao,
            an,
            old.4.get(owner).map(|l| l.len()),
            new.4.get(owner).map(|l| l.len()),
        );
        if only.is_some() {
            for id in ids_new.union(&ids_old) {
                println!(
                    "   {id} box old {:?}\n            new {:?}\n      line old {:?}\n           new {:?}",
                    old.6.get(id),
                    new.6.get(id),
                    old.2.get(id),
                    new.2.get(id)
                );
            }
        }
    }
    Ok(())
}
