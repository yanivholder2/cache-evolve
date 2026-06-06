//! Per-op-time fitness plugin for the real `TinyLfuCache`.
//!
//! Drives the live `TinyLfuCache<i64, i64>` through the `framework::Cache` trait so
//! variant edits to the cache impl (including the frequency sketch) show up in the
//! measured latencies. Mirrors the dash bench methodology: untimed warmup, fixed-count
//! timed get/put samples, p50/p99 via sorting, a deterministic xorshift64 key stream
//! over a range larger than the capacity (so the workload mixes hits, misses, and
//! evictions — and exercises TinyLfu's frequency-based admission on full-cache puts).
//!
//! Emits one plugin-shape JSON record on stdout:
//! `{ "dim": "per-op-time", "values": [{"op","p50_ns","p99_ns"},...], "summary": <get p50 ns>, "higher_is_better": false }`.

use std::time::{Duration, Instant};

use framework::Cache;
use tiny_lfu::fitness_json::write_plugin_json;
use tiny_lfu::tiny_lfu::TinyLfuCache;

/// Untimed — excludes cold-start cost (branch predictors, allocator, CPU caches) from p50/p99.
const WARMUP: usize = 500;

/// 2 000 × 1% = 20 tail samples for p99; small enough to stay sub-second per run.
const TIMED_ITERS: usize = 2_000;

/// Fixed capacity. Small so the working set fits in L1 — measures algorithmic cost,
/// not memory-hierarchy effects.
const CAP: usize = 1024;

/// Must exceed `CAP` so the workload produces a mix of hits, misses, and evictions.
const KEY_RANGE: u64 = 2 * CAP as u64;

/// Divisor for prefill count — half-fill so warmup starts with a mix of hits and misses.
const PREFILL_FRACTION_DIV: usize = 2;

/// Fixed PRNG seed: reproducible sample timings across runs and variants, so p50/p99
/// differences reflect cache changes, not RNG noise.
const RNG_SEED: u64 = 0xdeadbeef_cafebabe;

const P50_PCT: f64 = 50.0;
const P99_PCT: f64 = 99.0;

/// Returns the `pct`-th percentile (0–100) of `samples` in nanoseconds.
fn percentile(samples: &mut [Duration], pct: f64) -> u64 {
	assert!(!samples.is_empty(), "percentile called with empty samples");
	samples.sort_unstable();
	let idx = ((samples.len() as f64 * pct / 100.0) as usize).min(samples.len() - 1);
	samples[idx].as_nanos() as u64
}

/// Times `TIMED_ITERS` calls to `f`, preceded by `WARMUP` untimed iterations.
fn measure<F: FnMut()>(mut f: F) -> Vec<Duration> {
	for _ in 0..WARMUP {
		f();
	}
	let mut samples = Vec::with_capacity(TIMED_ITERS);
	for _ in 0..TIMED_ITERS {
		let t = Instant::now();
		f();
		samples.push(t.elapsed());
	}
	samples
}

/// Deterministic xorshift64 key stream mapped to `[1, KEY_RANGE]`. Same seed always
/// replays the same sequence; three shifts + one mod is a handful of ns, so it does
/// not contaminate samples and avoids pulling in `rand`.
fn next_key(state: &mut u64) -> i64 {
	*state ^= *state << 13;
	*state ^= *state >> 7;
	*state ^= *state << 17;
	(*state % KEY_RANGE + 1) as i64
}

fn main() {
	let mut cache = TinyLfuCache::<i64, i64>::new(CAP);
	let mut rng: u64 = RNG_SEED;

	for i in 0..(CAP / PREFILL_FRACTION_DIV) as i64 {
		Cache::put(&mut cache, i + 1, i);
	}

	// Pre-generate keys so RNG cost is excluded from per-op samples.
	let get_keys: Vec<i64> = (0..WARMUP + TIMED_ITERS).map(|_| next_key(&mut rng)).collect();
	let put_keys: Vec<i64> = (0..WARMUP + TIMED_ITERS).map(|_| next_key(&mut rng)).collect();

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

	// `summary` is get-p50: single representative latency for Pareto sorting.
	let summary = get_p50;

	let values = format!(
		r#"[
    {{"op": "get", "p50_ns": {get_p50}, "p99_ns": {get_p99}}},
    {{"op": "put", "p50_ns": {put_p50}, "p99_ns": {put_p99}}}
  ]"#,
	);
	print!("{}", write_plugin_json("per-op-time", &values, summary, false));
}
