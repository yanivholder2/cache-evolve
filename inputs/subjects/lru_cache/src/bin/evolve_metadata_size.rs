//! Static metadata-size measurement for fitness plugin consumption.
//!
//! Emits one plugin-shape JSON record:
//! `{ "dim": "metadata-size", "values": [{"type","bytes"},...], "summary": <Node bytes>, "higher_is_better": false }`.
//!
//! `summary` is `mem::size_of::<Node<u64,u64>>()` — the per-entry arena node overhead
//! that scales with cache population.
use lru_cache::lru_cache::{LruCache, Node};
use std::mem;

fn main() {
	let entries = [
		("Node<u64,u64>", mem::size_of::<Node<u64, u64>>()),
		("LruCache<u64,u64>", mem::size_of::<LruCache<u64, u64>>()),
	];

	let values_inner: Vec<String> = entries
		.iter()
		.map(|(name, bytes)| format!(r#"{{"type":"{name}","bytes":{bytes}}}"#))
		.collect();
	let values = format!("[{}]", values_inner.join(","));
	let summary = mem::size_of::<Node<u64, u64>>();

	print!("{}", lru_cache::fitness_json::write_plugin_json("metadata-size", &values, summary, false));
}
