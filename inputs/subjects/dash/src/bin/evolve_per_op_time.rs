//! Per-op-time measurement for fitness plugin consumption.
//!
//! Drives the REAL dash cache built through `factory::make_cache` — the same
//! factory the eval/JNI path uses — so variant overlay edits (e.g. geometry)
//! show up in the timings. Measures p50/p99 ns for `get` and `put` and emits one
//! plugin-shape JSON record on stdout:
//! `{ "dim": "per-op-time", "values": [{"op","p50_ns","p99_ns"},...], "summary": <get p50 ns>, "higher_is_better": false }`.
//!
//! The timing loop, key stream, and JSON writer come from `framework::bench`; the
//! cache under test is the real boxed `framework::Cache<i64,i64>`.

use framework::bench::{measure, next_key, percentile, write_plugin_json, P50_PCT, P99_PCT, RNG_SEED, TIMED_ITERS, WARMUP};

use dash::dash::dash_settings::DashSettings;

fn main() {
	// Default topology: 1 segment × (28 normal + 4 stash) buckets × 16 slots = 512-slot capacity.
	// Fixed so timings are comparable across runs; the geometry seam may refactor these fields,
	// but `make_cache` conserves the total budget. Keys range over 2× capacity so the workload
	// produces a steady mix of hits, misses, and evictions; the cache is half pre-filled before warmup.
	let settings = DashSettings::default();
	let capacity = settings.capacity() as u64;
	let key_range = capacity * 2;

	let mut cache = dash::factory::make_cache(&settings);
	let mut rng: u64 = RNG_SEED;

	for i in 0..capacity / 2 {
		cache.put(i as i64 + 1, i as i64);
	}

	// Pre-generate keys so RNG cost is excluded from per-op samples.
	let get_keys: Vec<i64> = (0..WARMUP + TIMED_ITERS).map(|_| next_key(&mut rng, key_range)).collect();
	let put_keys: Vec<i64> = (0..WARMUP + TIMED_ITERS).map(|_| next_key(&mut rng, key_range)).collect();

	let mut gi = 0usize;
	let mut get_samples = measure(|| {
		let k = get_keys[gi];
		gi += 1;
		cache.get(&k);
	});

	let mut pi = 0usize;
	let mut put_samples = measure(|| {
		let k = put_keys[pi];
		pi += 1;
		cache.put(k, k * 2);
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
