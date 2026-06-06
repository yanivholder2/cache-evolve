//! Per-op-time measurement for fitness plugin consumption.
//!
//! Drives the REAL dash cache built through `factory::make_cache` — the same
//! factory the eval/JNI path uses — so variant overlay edits (e.g. geometry)
//! show up in the timings. Measures p50/p99 ns for `get` and `put` and emits one
//! plugin-shape JSON record on stdout:
//! `{ "dim": "per-op-time", "values": [{"op","p50_ns","p99_ns"},...], "summary": <get p50 ns>, "higher_is_better": false }`.
//!
//! Methodology is carried over verbatim from the former standalone `bench/` crate,
//! except the cache under test is the real boxed `framework::Cache<i64,i64>`
//! instead of a fake fixture.

use std::time::{Duration, Instant};

use dash::dash::dash_settings::DashSettings;
use dash::fitness_json::write_plugin_json;

/// Untimed — excluded from samples so cold-start cost (branch predictors, allocator, CPU caches) doesn't skew p50/p99.
const WARMUP: usize = 500;

/// 2 000 × 1% = 20 tail samples for p99; small enough to stay sub-second per run.
const TIMED_ITERS: usize = 2_000;

/// Fixed PRNG seed: reproducible sample timings across runs and variants.
const RNG_SEED: u64 = 0xdeadbeef_cafebabe;

/// Percentile points reported in the output.
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

/// Deterministic key stream for the bench workload, mapped to `[1, KEY_RANGE]`.
///
/// Uses xorshift64, a PRNG (pseudo-random number generator) — an algorithm whose output looks random
/// but is fully determined by its seed, so the same seed always replays the same sequence. Needed because:
/// - **Workload shape:** random-ish keys over a range larger than the cache capacity force a realistic mix of
///   hits, misses, and evictions; sequential keys would degenerate into a predictable pattern.
/// - **Reproducibility:** a fixed seed means p50/p99 differences across variants reflect cache changes,
///   not RNG noise — essential for fitness comparison.
/// - **Cheap + std-only:** three shifts + one mod is a handful of ns, so it doesn't contaminate samples,
///   and avoids pulling in `rand`.
fn next_key(state: &mut u64, key_range: u64) -> i64 {
	*state ^= *state << 13;
	*state ^= *state >> 7;
	*state ^= *state << 17;
	(*state % key_range + 1) as i64
}

// ── main ──────────────────────────────────────────────────────────────────────

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

	// `summary` is get-p50: single representative latency for Pareto sorting.
	let summary = get_p50;

	let values = format!(
		r#"[{{"op":"get","p50_ns":{get_p50},"p99_ns":{get_p99}}},{{"op":"put","p50_ns":{put_p50},"p99_ns":{put_p99}}}]"#,
	);
	println!("{}", write_plugin_json("per-op-time", &values, summary, false));
}
