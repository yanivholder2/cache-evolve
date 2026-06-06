//! Per-op-time measurement for fitness plugin consumption.
//!
//! Drives the REAL cache built through `factory::make_cache` (the same seam the
//! eval/JNI path uses), so variant overlay edits to `src/shared/geometry.rs` show
//! up in the timings. The timing loop, key stream, and JSON writer come from
//! `framework::bench`.
//!
//! Emits one plugin-shape JSON record on stdout:
//! `{ "dim": "per-op-time", "values": [{"op","p50_ns","p99_ns"},...], "summary": <get p50 ns>, "higher_is_better": false }`.

use framework::bench::{measure, next_key, percentile, write_plugin_json, P50_PCT, P99_PCT, RNG_SEED, TIMED_ITERS, WARMUP};

use associative_cache::associative_cache::associative_cache_settings::AssociativeCacheSettings;

/// Fixed total capacity (budget the geometry seam factors into buckets × slots).
/// Small so the working set fits in L1 — measures algorithmic cost, not memory-hierarchy effects.
const CACHE_CAP: usize = 64;

/// Must exceed `CACHE_CAP` so the workload produces a mix of hits, misses, and evictions.
const KEY_RANGE: u64 = 128;

/// Divisor for prefill count — half-fill so warmup starts with a mix of hits and misses possible.
const PREFILL_FRACTION_DIV: usize = 2;

/// Representative default settings with a fixed `CACHE_CAP` budget.
///
/// Starts from `AssociativeCacheSettings::default` (ClassicLRU eviction) and sets the total
/// capacity to `CACHE_CAP` via `with_geometry`; `factory::make_cache` re-factors that budget
/// through the overlayable geometry seam, so variant geometry edits reshape the real cache.
fn bench_settings() -> AssociativeCacheSettings {
	AssociativeCacheSettings::default().with_geometry(1, CACHE_CAP)
}

fn main() {
	let settings = bench_settings();
	let mut cache = associative_cache::factory::make_cache(&settings);
	let mut rng: u64 = RNG_SEED;

	for i in 0..CACHE_CAP / PREFILL_FRACTION_DIV {
		cache.put(i as i64 + 1, i as i64);
	}

	// Pre-generate keys so RNG cost is excluded from per-op samples.
	let get_keys: Vec<i64> = (0..WARMUP + TIMED_ITERS).map(|_| next_key(&mut rng, KEY_RANGE)).collect();
	let put_keys: Vec<i64> = (0..WARMUP + TIMED_ITERS).map(|_| next_key(&mut rng, KEY_RANGE)).collect();

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
