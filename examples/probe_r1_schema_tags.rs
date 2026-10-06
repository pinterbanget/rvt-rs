//! Print the schema tags of named classes in each file's own
//! `Formats/Latest`, one line per file.
//!
//! The headers several record readers search for embed a class tag: the
//! element-record marker uses `Outline` and `ElementParents` (RE-80), the
//! element-data header `CellList` (RE-111), and the chain header
//! `ElementHeader`. Tags shift from release to release, so a reader that
//! takes them from the file's own schema carries to a release it was not
//! measured on, where a hardcoded value would not.
//!
//! ```bash
//! cargo run --release --example probe_r1_schema_tags -- \
//!     --classes CellList,ElementHeader,Outline,ElementParents FILE.rvt...
//! # which class holds a tag, in each file
//! cargo run --release --example probe_r1_schema_tags -- --tag 0x09fb FILE.rvt...
//! ```

use rvt::RevitFile;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut classes = vec![
        "CellList".to_string(),
        "ElementHeader".to_string(),
        "Outline".to_string(),
        "ElementParents".to_string(),
    ];
    let mut files = Vec::new();
    let mut by_tag: Option<u16> = None;
    while let Some(arg) = args.next() {
        if arg == "--tag" {
            let raw = args.next().unwrap_or_default();
            by_tag = u16::from_str_radix(raw.trim_start_matches("0x"), 16).ok();
        } else if arg == "--classes" {
            let list = args.next().unwrap_or_default();
            classes = list.split(',').map(str::to_string).collect();
        } else {
            files.push(arg);
        }
    }
    for path in files {
        let mut rf = RevitFile::open(&path)?;
        let version = rf.basic_file_info().map(|b| b.version).unwrap_or(0);
        let schema = rf.schema_classes()?;
        if let Some(tag) = by_tag {
            let name = schema.by_tag(tag).map_or("-", |class| class.name.as_str());
            println!("{version}\t0x{tag:04x}={name}\t{path}");
            continue;
        }
        let tags: Vec<String> = classes
            .iter()
            .map(
                |name| match schema.classes.iter().find(|c| &c.name == name) {
                    Some(class) => format!("{name}=0x{:04x}", class.tag),
                    None => format!("{name}=-"),
                },
            )
            .collect();
        println!("{version}\t{}\t{path}", tags.join(" "));
    }
    Ok(())
}
