//! Per-op-time measurement for fitness plugin consumption.
//!
//! Drives the REAL cache built through `factory::make_cache` (the same seam the
//! eval/JNI path uses), so variant overlay edits to `src/shared/geometry.rs`
//! show up in the timings. Methodology mirrors `dash/bench/src/main.rs`:
//! untimed warmup, fixed-count timed samples, p50/p99 via sorting, a
//! deterministic xorshift64 key stream with a fixed seed, and separate get/put
//! sample sets.
//!
//! Emits one plugin-shape JSON record on stdout:
//! `{ "dim": "per-op-time", "values": [{"op","p50_ns","p99_ns"},...], "summary": <get p50 ns>, "higher_is_better": false }`.
use associative_cache::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use associative_cache::fitness_json::write_plugin_json;
use std::time::{Duration, Instant};

/// Untimed — excluded from samples so cold-start cost (branch predictors, allocator, CPU caches) doesn't skew p50/p99.
const WARMUP: usize = 500;

/// 2 000 × 1% = 20 tail samples for p99; small enough to stay sub-second per run.
const TIMED_ITERS: usize = 2_000;

/// Fixed total capacity (budget the geometry seam factors into buckets × slots).
/// Small so the working set fits in L1 — measures algorithmic cost, not memory-hierarchy effects.
const CACHE_CAP: usize = 64;

/// Must exceed `CACHE_CAP` so the workload produces a mix of hits, misses, and evictions.
const KEY_RANGE: i64 = 128;

/// Divisor for prefill count — half-fill so warmup starts with a mix of hits and misses possible.
const PREFILL_FRACTION_DIV: usize = 2;

/// Fixed PRNG seed: reproducible sample timings across runs and variants.
const RNG_SEED: u64 = 0xdeadbeef_cafebabe;

/// Percentile points reported in the record.
const P50_PCT: f64 = 50.0;
const P99_PCT: f64 = 99.0;

// ── measurement helpers ───────────────────────────────────────────────────────

/// Returns the `pct`-th percentile (0–100) of `samples` in nanoseconds; `pct = 100` yields the maximum.
fn percentile(samples: &mut [Duration], pct: f64) -> u64 {
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

/// Deterministic key stream mapped to `[1, KEY_RANGE]`.
///
/// Uses xorshift64, a PRNG (pseudo-random number generator) — fully determined by its seed, so the same
/// seed always replays the same sequence. A range larger than `CACHE_CAP` forces a realistic mix of hits,
/// misses, and evictions; the fixed seed means p50/p99 differences across variants reflect cache changes,
/// not RNG noise. Three shifts + one mod is a handful of ns, so it doesn't contaminate samples.
fn next_key(state: &mut u64) -> i64 {
	*state ^= *state << 13;
	*state ^= *state >> 7;
	*state ^= *state << 17;
	(*state % KEY_RANGE as u64) as i64 + 1
}

// ── settings ──────────────────────────────────────────────────────────────────

/// Representative default settings with a fixed `CACHE_CAP` budget.
///
/// Starts from `AssociativeCacheSettings::default` (ClassicLRU eviction) and sets the total
/// capacity to `CACHE_CAP` via `with_geometry`; `factory::make_cache` re-factors that budget
/// through the overlayable geometry seam, so variant geometry edits reshape the real cache.
fn bench_settings() -> AssociativeCacheSettings {
	AssociativeCacheSettings::default().with_geometry(1, CACHE_CAP)
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
	let settings = bench_settings();
	let mut cache = associative_cache::factory::make_cache(&settings);
	let mut rng: u64 = RNG_SEED;

	for i in 0..CACHE_CAP / PREFILL_FRACTION_DIV {
		cache.put(i as i64 + 1, i as i64);
	}

	// Pre-generate keys so RNG cost is excluded from per-op samples.
	let get_keys: Vec<i64> = (0..WARMUP + TIMED_ITERS).map(|_| next_key(&mut rng)).collect();
	let put_keys: Vec<i64> = (0..WARMUP + TIMED_ITERS).map(|_| next_key(&mut rng)).collect();

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

	// `summary` is get-p50: single representative latency for Pareto sorting.
	let summary = get_p50;

	let values = format!(
		r#"[{{"op":"get","p50_ns":{get_p50},"p99_ns":{get_p99}}},{{"op":"put","p50_ns":{put_p50},"p99_ns":{put_p99}}}]"#,
	);

	println!("{}", write_plugin_json("per-op-time", &values, summary, false));
}
