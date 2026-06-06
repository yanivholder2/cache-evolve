//! Static metadata-size measurement for fitness plugin consumption.
//!
//! Emits one plugin-shape JSON record:
//! `{ "dim": "metadata-size", "values": [{"type","bytes"},...], "summary": <bytes>, "higher_is_better": false }`.
//!
//! `summary` is `mem::size_of::<S3Fifo<u64,u64>>()` — the cache's fixed control-block
//! footprint; full-autonomy variants that grow the struct's fields pay for it here.
use s3_fifo::s3_fifo::S3Fifo;
use std::mem;

/// Format a plugin JSON record (std-only; mirrors the bench crate's writer).
fn write_plugin_json(dim: &str, values_json: &str, summary: usize, higher_is_better: bool) -> String {
    format!(
        "{{\n  \"dim\": \"{dim}\",\n  \"values\": {values_json},\n  \"summary\": {summary},\n  \"higher_is_better\": {higher_is_better}\n}}\n",
    )
}

fn main() {
    let entries = [("S3Fifo<u64,u64>", mem::size_of::<S3Fifo<u64, u64>>())];

    let values_inner: Vec<String> = entries
        .iter()
        .map(|(name, bytes)| format!(r#"{{"type":"{name}","bytes":{bytes}}}"#))
        .collect();
    let values = format!("[{}]", values_inner.join(","));
    let summary = mem::size_of::<S3Fifo<u64, u64>>();

    print!("{}", write_plugin_json("metadata-size", &values, summary, false));
}
