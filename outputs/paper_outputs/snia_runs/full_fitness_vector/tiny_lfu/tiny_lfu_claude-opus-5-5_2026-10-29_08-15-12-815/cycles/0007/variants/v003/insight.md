# Variant v003 Insight

## Helped

- hit-rate (hm_0 @ 16384): This cell gained the most. At the small size the 819-entry window turns over fast, and newcomers used to arrive with a double-counted estimate. That let one-time keys beat probation victims in `evict_one` too often. Skipping the duplicate `put` increment makes the contest more honest here, which is the effect the hypothesis predicted, but it shows up at the small size, not the large one.
- hit-rate (web_0 @ 16384): A marginal gain from the same fix. Fewer inflated one-time candidates are admitted over residents.

## Hurt

- hit-rate (web_0 @ 131072): This is the largest regression, and it drives the drop in the summary. There are two side effects, and this data cannot separate them. (1) Each miss now advances `additions` once instead of twice, so on this miss-heavy trace the 10×capacity aging period stretches out. Stale counts build up between resets. (2) The +1 head start on a miss was working as a recency bonus that let the large window's candidates get admitted. With conservative update already under-counting, removing it leaves candidates tied with victims. Ties go to the victim (`cand_freq > victim_freq` is strict), so recency-driven web_0 keys get rejected more often.
- hit-rate (hm_0 @ 131072): A small further dip in the cell v001 had already hurt. It moved in the opposite direction from the hypothesis's main prediction. Removing the overcount did not fix the large-size weakness. It made that weakness slightly worse.
- hit-rate (summary): The large-size losses outweigh the small-size gains, so the hypothesis's target was missed.
- per-op-time (get/put p50 and p99): Every percentile got much slower, even though the change only adds a compare, a branch and two field writes per op. The slowdown is far too large for that much work. The likely causes are codegen changes: `get` replaced the `?` early return with a `match` that writes `self` on both arms, and `put` gained a branch in front of the inlined sketch body. Either could change inlining or layout. Benchmark noise is also possible. Either way, this variant is a latency regression as measured.
- metadata-size (TinyLfuCache fixed struct): The struct grew by one cache-line-irrelevant field pair (u64 + bool, padded). The node size and sketch bytes are unchanged, so the summary dimension did not move.

## Takeaway

On this driver (miss → `get` + `put`), the "double count" on first touch is not just noise. Paired with v001's pairwise conservative update and strict `>` admission test, it acts as a small admission bias toward newcomers, plus a faster aging clock. The large-size cells, web_0 in particular, depended on both. Removing it improves small-cache accuracy but hurts large-cache hit-rate. A follow-up should restore the aging rate on its own (e.g. count skipped increments toward `additions`, or lower `SAMPLE_FACTOR`). It should also re-benchmark latency, to check whether the per-op slowdown comes from the `get` restructuring rather than the dedup logic itself.
