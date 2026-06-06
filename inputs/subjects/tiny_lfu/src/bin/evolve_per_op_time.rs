//! Per-op-time fitness plugin for the real `TinyLfuCache`.
//!
//! Drives the live `TinyLfuCache<i64, i64>` through the `framework::Cache` trait so
//! variant edits to the cache impl (including the frequency sketch) show up in the
//! measured latencies — the key stream ranges over 2× capacity so full-cache puts
//! exercise TinyLfu's frequency-based admission. The timing loop, key stream, and
//! JSON writer come from `framework::bench`.
//!
//! Emits one plugin-shape JSON record on stdout:
//! `{ "dim": "per-op-time", "values": [{"op","p50_ns","p99_ns"},...], "summary": <get p50 ns>, "higher_is_better": false }`.

use framework::bench::{measure, next_key, percentile, write_plugin_json, P50_PCT, P99_PCT, RNG_SEED, TIMED_ITERS, WARMUP};
use framework::Cache;
use tiny_lfu::tiny_lfu::TinyLfuCache;

/// Fixed capacity. Small so the working set fits in L1 — measures algorithmic cost,
/// not memory-hierarchy effects.
const CAP: usize = 1024;

/// Must exceed `CAP` so the workload produces a mix of hits, misses, and evictions.
const KEY_RANGE: u64 = 2 * CAP as u64;

/// Divisor for prefill count — half-fill so warmup starts with a mix of hits and misses.
const PREFILL_FRACTION_DIV: usize = 2;

fn main() {
	let mut cache = TinyLfuCache::<i64, i64>::new(CAP);
	let mut rng: u64 = RNG_SEED;

	for i in 0..(CAP / PREFILL_FRACTION_DIV) as i64 {
		Cache::put(&mut cache, i + 1, i);
	}

	// Pre-generate keys so RNG cost is excluded from per-op samples.
	let get_keys: Vec<i64> = (0..WARMUP + TIMED_ITERS).map(|_| next_key(&mut rng, KEY_RANGE)).collect();
	let put_keys: Vec<i64> = (0..WARMUP + TIMED_ITERS).map(|_| next_key(&mut rng, KEY_RANGE)).collect();

	let mut gi = 0usize;
	let mut get_samples = measure(|| {
		let k = get_keys[gi];
		gi += 1;
		let _ = Cache::get(&mut cache, &k);
	});

	let mut pi = 0usize;
	let mut put_samples = measure(|| {
		let k = put_keys[pi];
		pi += 1;
		Cache::put(&mut cache, k, k * 2);
	});

	let get_p50 = percentile(&mut get_samples, P50_PCT);
	let get_p99 = percentile(&mut get_samples, P99_PCT);
	let put_p50 = percentile(&mut put_samples, P50_PCT);
	let put_p99 = percentile(&mut put_samples, P99_PCT);

	// `summary` is get-p50: single representative latency for selection sorting.
	let summary = get_p50;

	let values = format!(
		r#"[{{"op":"get","p50_ns":{get_p50},"p99_ns":{get_p99}}},{{"op":"put","p50_ns":{put_p50},"p99_ns":{put_p99}}}]"#,
	);
	println!("{}", write_plugin_json("per-op-time", &values, summary, false));
}
