//! Static metadata-size measurement for fitness plugin consumption.
//!
//! Emits one plugin-shape JSON record:
//! `{ "dim": "metadata-size", "values": [{"type","bytes"},...], "summary": <Entry bytes>, "higher_is_better": false }`.
//!
//! `summary` is `mem::size_of::<Entry<u64,u64>>()` — the per-entry overhead that scales with
//! cache population. S3-FIFO holds entries across its small and main queues, but the per-entry
//! `size_of` is still the right per-element metric.

use framework::bench::write_plugin_json;
use s3_fifo::s3_fifo::{Entry, S3Fifo};
use std::mem;

fn main() {
	let entries = [
		("Entry<u64,u64>", mem::size_of::<Entry<u64, u64>>()),
		("S3Fifo<u64,u64>", mem::size_of::<S3Fifo<u64, u64>>()),
	];

	let values_inner: Vec<String> = entries
		.iter()
		.map(|(name, bytes)| format!(r#"{{"type":"{name}","bytes":{bytes}}}"#))
		.collect();
	let values = format!("[{}]", values_inner.join(","));
	let summary = mem::size_of::<Entry<u64, u64>>();

	println!("{}", write_plugin_json("metadata-size", &values, summary, false));
}
