# Variant v002 Insight

## Helped

- per-op-time: Both `get` and `put` got faster at p50 and p99, but the code change can't explain it. `increment_hash` does strictly more work than in v003: it still runs the same two-pass conservative update, then adds an `additions` increment, a compare against `sample_size`, and a rare `#[cold]` `reset()` that sweeps all 64 KiB. The best the change can claim is that the `#[cold] #[inline(never)]` split kept the hot path from getting slower. The improvement itself is most likely run-to-run environment variance, which earlier insights already flagged for this bench. Don't credit aging with a speedup.

## Hurt

- hit-rate: Halving every counter every `10 × capacity` sketch increments wears down the long-lived frequency lead that resident keys build up. Gets count toward the sample too, so with capacity 2048 a reset happens every 20,480 accesses. Admission compares the incoming key against the LRU-head victim, so that victim's estimate keeps getting pulled back toward the cold newcomers' level. Small counts also collapse after a reset (1→0, 2–3→1). That turns earlier gaps into ties, or lets a key that has just been bumped to 1 beat a victim that was halved to 0. Either way, the strict `>` admission rule loses the margin it relied on to keep scan or loop traffic out. Workload-a evidently rewards stable long-term frequency more than quick adaptation, so decay discarded exactly the signal that made conservative update pay off in v003.
- metadata-size: `TinyLfuCache<u64,u64>` grows by the two new `u32` fields (`additions`, `sample_size`). `Node` stays at 24 bytes and the sketch heap is unchanged, so the summary dimension doesn't move. Only the fixed per-cache footprint grows slightly.

## Takeaway

On workload-a, adding Caffeine-style halving to the sketch clearly hurts hit-rate, even with full-width `u32` counters and conservative update. Cycle 0002's aging failure was therefore not only a side effect of its 4-bit cap. Decay itself is harmful here as long as admission is a strict frequency duel against the LRU head with no admission window. The hypothesis's own criterion (beat v003's hit-rate) rejects it. If aging is tried again, use a much longer reset period, or exclude `get` hits on resident keys from the sample count. A blanket 10×capacity halving should not be retried.
