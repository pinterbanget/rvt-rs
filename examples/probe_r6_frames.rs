//! Print a 32-bit file's record marker and, for each ElementId given, every
//! place its `u32` occurs in a partition with the nearest marker after it.
//!
//! ```bash
//! cargo run --release --example probe_r6_frames -- FILE.rvt ID...
//! ```

use rvt::RevitFile;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut rf = RevitFile::open(&args[0])?;
    let (marker, header) = rvt::partition_element_records_2023::record_marker(&mut rf).unwrap();
    println!("marker {} header tag {header:#06x}", hex(&marker));
    let ids: Vec<u32> = args[1..].iter().filter_map(|s| s.parse().ok()).collect();
    for stream in rf.partition_stream_names() {
        let Ok(inflated) = rf.inflated_partition(&stream) else {
            continue;
        };
        let buf = inflated.bytes();
        let markers: Vec<usize> = memchr::memmem::find_iter(buf, &marker).collect();
        for id in &ids {
            for at in memchr::memmem::find_iter(buf, &id.to_le_bytes()) {
                let next = markers.iter().find(|&&m| m > at && m - at < 600);
                if let Some(&m) = next {
                    let from = at.saturating_sub(16);
                    println!(
                        "{stream} id {id} at {at}, marker +{}\n  {}",
                        m - at,
                        hex(&buf[from..(m + 8 + 48 + 40).min(buf.len())])
                    );
                }
            }
        }
    }
    Ok(())
}

fn hex(b: &[u8]) -> String {
    b.chunks(4)
        .map(|c| c.iter().map(|x| format!("{x:02x}")).collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}
