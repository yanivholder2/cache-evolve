//! Static + runtime metadata-size measurement for fitness plugin consumption.
//!
//! Emits one plugin-shape JSON record:
//! `{ "dim": "metadata-size", "values": [{"type","bytes"},...], "summary": <Node bytes>, "higher_is_better": false }`.
//!
//! TinyLfu's metadata has two parts:
//!  - per-entry node overhead: `mem::size_of::<Node<u64,u64>>()` — scales with cache *population*;
//!  - the frequency-sketch overhead: a `CountMinSketch` whose counter-table heap allocation is sized
//!    from *capacity* at construction (`width = next_pow2(max(capacity*4, 16))`, `depth = 4`,
//!    `u32` counters), independent of how many entries are live.
//!
//! `summary` is the per-entry node byte size — the population-scaling component, consistent with the
//! other subjects (associative_cache reports `Item`, lru_cache reports `Node`). The sketch heap size
//! is only knowable at runtime (it depends on capacity), so we construct a real
//! `TinyLfuCache::<u64,u64>::new(CAP)` and read its `sketch_heap_bytes()` accessor; both the sketch
//! bytes and the whole-struct `size_of` are reported in `values` so the fixed overhead stays visible.
use framework::bench::write_plugin_json;
use std::mem;
use tiny_lfu::tiny_lfu::{Node, TinyLfuCache};

/// Capacity used to materialise the sketch for its runtime heap measurement.
/// Matches the per-op-time bin's CAP so the two dimensions describe the same cache shape.
const CAP: usize = 1024;

fn main() {
	let probe = TinyLfuCache::<u64, u64>::new(CAP);
	let sketch_heap_bytes = probe.sketch_heap_bytes();

	let entries = [
		("Node<u64,u64>", mem::size_of::<Node<u64, u64>>()),
		("CountMinSketch (heap, cap=1024)", sketch_heap_bytes),
		("TinyLfuCache<u64,u64>", mem::size_of::<TinyLfuCache<u64, u64>>()),
	];

	let values_inner: Vec<String> = entries
		.iter()
		.map(|(name, bytes)| format!(r#"{{"type":"{name}","bytes":{bytes}}}"#))
		.collect();
	let values = format!("[{}]", values_inner.join(","));
	let summary = mem::size_of::<Node<u64, u64>>();

	println!("{}", write_plugin_json("metadata-size", &values, summary, false));
}
