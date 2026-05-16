//! Microbench harness — std-only, single-threaded.
//! Measures p50/p99 ns for `get` and `put`, writes `bench.json` in per-op-time fitness-plugin shape.

mod cache_fixture;
mod fitness_json;

use std::{
	env, fs,
	time::{Duration, Instant},
};

use cache_fixture::CacheFixture;
use fitness_json::write_plugin_json;

/// Untimed — excluded from samples so cold-start cost (branch predictors, allocator, CPU caches) doesn't skew p50/p99.
const WARMUP: usize = 500;

/// 2 000 × 1% = 20 tail samples for p99; small enough to stay sub-second per run.
const TIMED_ITERS: usize = 2_000;

/// Small so the working set fits in L1 — measures algorithmic cost, not memory-hierarchy effects.
const CACHE_CAP: usize = 64;

/// Must exceed `CACHE_CAP` so the workload produces a mix of hits, misses, and evictions.
const KEY_RANGE: u64 = 128;

/// Divisor for prefill count — half-fill so warmup starts with a mix of hits and misses possible.
const PREFILL_FRACTION_DIV: usize = 2;

/// Fixed PRNG seed: reproducible sample timings across runs and variants.
const RNG_SEED: u64 = 0xdeadbeef_cafebabe;

/// Percentile points reported in `bench.json`.
const P50_PCT: f64 = 50.0;
const P99_PCT: f64 = 99.0;

// ── measurement helpers ───────────────────────────────────────────────────────

/// Returns the `pct`-th percentile (0–100) of `samples` in nanoseconds; `pct = 100` yields the maximum.
fn percentile(samples: &mut Vec<Duration>, pct: f64) -> u64 {
	assert!(!samples.is_empty(), "percentile called with empty samples");
	samples.sort_unstable();
	let idx = ((samples.len() as f64 * pct / 100.0) as usize).min(samples.len() - 1);
	samples[idx].as_nanos() as u64
}

/// Times `TIMED_ITERS` calls to `f`, preceded by `WARMUP` untimed iterations; keeps all samples so p99 stays meaningful.
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

/// Deterministic key stream for the bench workload, mapped to `[1, KEY_RANGE]`.
///
/// Uses xorshift64, a PRNG (pseudo-random number generator) — an algorithm whose output looks random
/// but is fully determined by its seed, so the same seed always replays the same sequence. Needed because:
/// - **Workload shape:** random-ish keys over a range larger than `CACHE_CAP` force a realistic mix of
///   hits, misses, and evictions; sequential keys would degenerate into a predictable pattern.
/// - **Reproducibility:** a fixed seed means p50/p99 differences across variants reflect cache changes,
///   not RNG noise — essential for fitness comparison.
/// - **Cheap + std-only:** three shifts + one mod is a handful of ns, so it doesn't contaminate samples,
///   and avoids pulling in `rand`.
fn next_key(state: &mut u64) -> u64 {
	*state ^= *state << 13;
	*state ^= *state >> 7;
	*state ^= *state << 17;
	*state % KEY_RANGE + 1
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
	let args: Vec<String> = env::args().collect();
	let output = match args.iter().position(|a| a == "--output") {
		Some(i) => args.get(i + 1).cloned().unwrap_or_else(|| {
			eprintln!("error: --output requires a path argument");
			std::process::exit(2);
		}),
		None => "bench.json".to_string(),
	};

	let mut cache = CacheFixture::new(CACHE_CAP);
	let mut rng: u64 = RNG_SEED;

	for i in 0..CACHE_CAP / PREFILL_FRACTION_DIV {
		cache.put(i as u64 + 1, i as u64);
	}

	// Pre-generate keys so RNG cost is excluded from per-op samples.
	let get_keys: Vec<u64> = (0..WARMUP + TIMED_ITERS).map(|_| next_key(&mut rng)).collect();
	let put_keys: Vec<u64> = (0..WARMUP + TIMED_ITERS).map(|_| next_key(&mut rng)).collect();

	let mut gi = 0usize;
	let mut get_samples = measure(|| {
		let k = get_keys[gi];
		gi += 1;
		cache.get(k);
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

	// `summary` is get-p50: single representative latency for Pareto sorting.
	let summary = get_p50;

	let values = format!(
		r#"[
    {{"op": "get", "p50_ns": {get_p50}, "p99_ns": {get_p99}}},
    {{"op": "put", "p50_ns": {put_p50}, "p99_ns": {put_p99}}}
  ]"#,
	);
	let json = write_plugin_json("per-op-time", &values, summary, false);

	fs::write(&output, &json).unwrap_or_else(|e| {
		eprintln!("error writing {output}: {e}");
		std::process::exit(1);
	});
	print!("{json}");
}
