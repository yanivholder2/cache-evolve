//! Shared, std-only measurement harness for the subjects' `evolve-per-op-time`
//! and `evolve-metadata-size` bins.
//!
//! Lives in the framework crate — not in any subject — so every subject's bin
//! reuses one copy of the timing loop, key stream, and JSON writer. Because the
//! framework crate is referenced by absolute path and is never part of a variant
//! overlay, a variant cannot edit this harness to fake its own fitness.

use std::fmt::Display;
use std::time::{Duration, Instant};

/// Untimed iterations run before sampling, so cold-start cost (branch predictors,
/// allocator, CPU caches) doesn't skew p50/p99.
pub const WARMUP: usize = 500;

/// Timed samples kept per op. 2 000 × 1% = 20 tail samples for p99; small enough
/// to stay sub-second per run.
pub const TIMED_ITERS: usize = 2_000;

/// Percentile points reported by the per-op-time bins.
pub const P50_PCT: f64 = 50.0;
pub const P99_PCT: f64 = 99.0;

/// Fixed PRNG seed: reproducible sample timings across runs and variants, so
/// p50/p99 differences reflect cache changes, not RNG noise.
pub const RNG_SEED: u64 = 0xdeadbeef_cafebabe;

/// Returns the `pct`-th percentile (0–100) of `samples` in nanoseconds; `pct = 100` yields the maximum.
pub fn percentile(samples: &mut [Duration], pct: f64) -> u64 {
	assert!(!samples.is_empty(), "percentile called with empty samples");
	samples.sort_unstable();
	let idx = ((samples.len() as f64 * pct / 100.0) as usize).min(samples.len() - 1);
	samples[idx].as_nanos() as u64
}

/// Times `TIMED_ITERS` calls to `f`, preceded by `WARMUP` untimed iterations; keeps
/// all samples so p99 stays meaningful.
pub fn measure<F: FnMut()>(mut f: F) -> Vec<Duration> {
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

/// Deterministic xorshift64 key stream mapped to `[1, key_range]`.
///
/// xorshift64 is a PRNG (pseudo-random number generator) fully determined by its seed,
/// so the same seed always replays the same sequence — p50/p99 differences across
/// variants reflect cache changes, not RNG noise. A `key_range` larger than the cache
/// capacity forces a realistic mix of hits, misses, and evictions. Three shifts plus one
/// mod is a handful of ns, so it doesn't contaminate samples and avoids pulling in `rand`.
pub fn next_key(state: &mut u64, key_range: u64) -> i64 {
	*state ^= *state << 13;
	*state ^= *state >> 7;
	*state ^= *state << 17;
	(*state % key_range + 1) as i64
}

/// Format a plugin JSON record. `values_json` must already be a well-formed JSON array literal.
///
/// Contract: `{ "dim": <str>, "values": <json-array>, "summary": <number>, "higher_is_better": <bool> }`.
pub fn write_plugin_json(dim: &str, values_json: &str, summary: impl Display, higher_is_better: bool) -> String {
	assert!(
		dim.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
		"dim must match [a-z0-9-]+, got {dim:?}",
	);
	format!(
		"{{\"dim\":\"{dim}\",\"values\":{values_json},\"summary\":{summary},\"higher_is_better\":{higher_is_better}}}",
	)
}
