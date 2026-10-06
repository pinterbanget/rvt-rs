//! For each record group of channels 101 to 103 in a file's partitions, try
//! record-header lengths and report which one makes the records the group
//! declares (count and body bytes) read out exactly, by channel.
//!
//! A record is `[id][header rest][u32 length]`, the body, and the length
//! again; `native_document::group_records` takes the header as the id width
//! plus 4 bytes on channel 101 and plus 8 on 102 and 103.
//!
//! ```bash
//! cargo run --release --example probe_r3_group_headers -- FILE.rvt
//! ```

use rvt::{RevitFile, compression, native_segments, schema_registry};
use std::collections::BTreeMap;

fn fits(bytes: &[u8], id_bytes: usize, header: usize, objects: u64, body: u64) -> bool {
    let (mut pos, mut count, mut sum) = (0usize, 0u64, 0u64);
    while pos < bytes.len() {
        if bytes.len() - pos < header + 4 {
            return false;
        }
        let Some(len) = bytes
            .get(pos + header - 4..pos + header)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize)
        else {
            return false;
        };
        let end = pos + header + len;
        if end + 4 > bytes.len()
            || u32::from_le_bytes(bytes[end..end + 4].try_into().unwrap()) as usize != len
        {
            return false;
        }
        let _ = id_bytes;
        count += 1;
        sum += len as u64;
        pos = end + 4;
    }
    count == objects && sum == body
}

/// Read records with `header` until the group ends or a record breaks;
/// `(records, body bytes, bytes consumed)`.
fn walk_records(bytes: &[u8], header: usize) -> (u64, u64, usize) {
    let (mut pos, mut count, mut sum) = (0usize, 0u64, 0u64);
    while bytes.len() - pos >= header + 4 {
        let len =
            u32::from_le_bytes(bytes[pos + header - 4..pos + header].try_into().unwrap()) as usize;
        let end = pos + header + len;
        if end + 4 > bytes.len()
            || u32::from_le_bytes(bytes[end..end + 4].try_into().unwrap()) as usize != len
        {
            break;
        }
        count += 1;
        sum += len as u64;
        pos = end + 4;
    }
    (count, sum, pos)
}

fn main() -> anyhow::Result<()> {
    for path in std::env::args().skip(1) {
        let mut rf = RevitFile::open(&path)?;
        let version = rf.basic_file_info()?.version;
        let formats = rf.read_stream("Formats/Latest")?;
        let registry = schema_registry::parse(&compression::inflate_stream_at(
            "Formats/Latest",
            &formats,
            0,
        )?)?;
        // (channel, header length) -> groups that read out; groups per channel
        let mut fit: BTreeMap<(u64, usize), usize> = BTreeMap::new();
        let mut groups: BTreeMap<u64, usize> = BTreeMap::new();
        let mut example: BTreeMap<u64, String> = BTreeMap::new();
        for name in rf.partition_stream_names() {
            let stored = rf.read_stream(&name)?;
            let prepared = compression::prepare_stream_for_inflate(&name, &stored);
            let walked = native_segments::walk(&prepared, &registry, 1 << 30, |source, bytes| {
                if source.content_key.is_some() || !(101..=103).contains(&source.channel) {
                    return Ok(());
                }
                *groups.entry(source.channel).or_default() += 1;
                if std::env::var_os("DUMP").is_some() && !example.contains_key(&source.channel) {
                    let n = bytes.len().min(400);
                    println!(
                        "  dump channel {} (objects {} body {}):",
                        source.channel, source.declared_objects, source.declared_body_bytes
                    );
                    for (row, chunk) in bytes[..n].chunks(32).enumerate() {
                        println!(
                            "    +{:03x} {}",
                            row * 32,
                            chunk
                                .iter()
                                .map(|b| format!("{b:02x}"))
                                .collect::<Vec<_>>()
                                .join(" ")
                        );
                    }
                }
                example.entry(source.channel).or_insert_with(|| {
                    format!(
                        "objects {} body {} first bytes {}",
                        source.declared_objects,
                        source.declared_body_bytes,
                        bytes
                            .iter()
                            .take(32)
                            .map(|b| format!("{b:02x}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                });
                if std::env::var_os("WALK").is_some()
                    && *groups.get(&source.channel).unwrap_or(&0) <= 3
                {
                    let header = if source.channel == 101 { 8 } else { 12 };
                    let (n, sum, pos) = walk_records(bytes, header);
                    println!(
                        "  ch {} group: declared {} objects / {} body; read {} / {} over {} of {} bytes; next {}",
                        source.channel,
                        source.declared_objects,
                        source.declared_body_bytes,
                        n,
                        sum,
                        pos,
                        bytes.len(),
                        bytes
                            .iter()
                            .skip(pos)
                            .take(24)
                            .map(|b| format!("{b:02x}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                }
                for header in [8usize, 12, 16, 20, 24] {
                    if fits(
                        bytes,
                        4,
                        header,
                        source.declared_objects,
                        source.declared_body_bytes,
                    ) {
                        *fit.entry((source.channel, header)).or_default() += 1;
                    }
                }
                Ok(())
            });
            if let Err(error) = walked {
                println!("  {name}: walk failed: {error:#}");
            }
        }
        println!("{version} {path}");
        for (channel, n) in &groups {
            let fits: Vec<String> = fit
                .iter()
                .filter(|((c, _), _)| c == channel)
                .map(|((_, h), k)| format!("header {h}: {k}"))
                .collect();
            println!("  channel {channel}: {n} groups; {}", fits.join(", "));
            println!("    e.g. {}", example[channel]);
        }
    }
    Ok(())
}
