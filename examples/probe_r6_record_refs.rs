//! Print the 32-bit element records of the given ElementIds: category,
//! class tag, box, container, reference list and owner.
//!
//! ```bash
//! cargo run --release --example probe_r6_record_refs -- FILE.rvt ID...
//! ```

use rvt::RevitFile;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut rf = RevitFile::open(&args[0])?;
    let version = rf.basic_file_info()?.version;
    let ids: Vec<u32> = args[1..].iter().filter_map(|s| s.parse().ok()).collect();
    for r in rvt::partition_element_records_2023::scan_records(&mut rf, version) {
        if ids.contains(&r.element_id) {
            println!(
                "{} {} cat {} class {:#06x} kind {:#x} container {} box {:?}\n   refs {:?} owner {:?}",
                r.element_id,
                r.stream,
                r.builtin_category,
                r.class_tag,
                r.placement_kind,
                r.container as i64,
                r.bbox_feet,
                r.references,
                r.owner_reference
            );
        }
    }
    for r in rvt::partition_element_records_2023::scan_unoutlined_records(&mut rf, version) {
        if ids.contains(&r.element_id) {
            println!(
                "{} {} (no outline) cat {} class {:#06x} kind {:#x}\n   refs {:?} owner {:?}",
                r.element_id,
                r.stream,
                r.builtin_category,
                r.class_tag,
                r.placement_kind,
                r.references,
                r.owner_reference
            );
        }
    }
    Ok(())
}
