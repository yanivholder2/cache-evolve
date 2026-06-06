//! Static metadata-size measurement for fitness plugin consumption.
//!
//! Emits one plugin-shape JSON record:
//! `{ "dim": "metadata-size", "values": [{"type","bytes"},...], "summary": <Item bytes>, "higher_is_better": false }`.
//!
//! `summary` is `mem::size_of::<Item<u64,u64>>()` — the per-entry overhead that scales with cache population.
use associative_cache::associative_cache::AssociativeCache;
use associative_cache::shared::item::Item;
use framework::bench::write_plugin_json;
use std::mem;

fn main() {
	let entries = [
		("Item<u64,u64>", mem::size_of::<Item<u64, u64>>()),
		("AssociativeCache<u64,u64>", mem::size_of::<AssociativeCache<u64, u64>>()),
	];

	let values_inner: Vec<String> = entries
		.iter()
		.map(|(name, bytes)| format!(r#"{{"type":"{name}","bytes":{bytes}}}"#))
		.collect();
	let values = format!("[{}]", values_inner.join(","));
	let summary = mem::size_of::<Item<u64, u64>>();

	println!("{}", write_plugin_json("metadata-size", &values, summary, false));
}
