# Variant v001 Insight

## Helped

- hit-rate (16384 cells, web_0 and hm_0): Both small-size cells went up. At this size `window_size` now gives a window of 819 entries where the parent's 1% rule gave 163, so recency-heavy bursts in these traces survive long enough in the window to build sketch frequency before they face the admission contest. Pairwise conservative update also adds less collision inflation to the main-region victims' estimates.
- hit-rate (web_0 @ 131072): This cell rose even though the window is bit-identical to the parent (1310 = pct1 beats the 820 cap, which `window_floor_sizes` confirms). The gain therefore comes only from the pairwise conservative update. Bumping only the pair-minimum counters keeps one-hit keys from inflating shared counters, so window candidates face more honest victim estimates.
- hit-rate (summary): The merge cleared the hypothesis's target and slightly beat its prediction that it would land on the floor-only result. The two levers add up on the 24-byte line without cancelling. This happened even though v001's odd-count-corrected `reset()` was kept unchanged.
- per-op-time (get p50, summary): Get p50 went down instead of up, even though `increment_hashed` now does four reads, two mins and four masked adds on every `get`/`put`. With `#[inline(always)]`, the branchless pairwise body folds into the call sites as straight-line ALU work on a block that is already in cache. It costs no more than the parent's plain increment did. Put p99 also eased.

## Hurt

- hit-rate (hm_0 @ 131072): A small dip in the one cell where only conservative update changed and hm_0 is already near its ceiling. Here conservative update's under-counting of keys that share counters slightly weakens their protection in the eviction contest.
- per-op-time (get p99): The get tail widened while the median improved. The likely cause is the larger inlined sketch body in `get` (more code and register pressure on the hit path), or `reset()` running at a different rate now that `additions` only counts live increments. The data cannot separate these two causes.

## Takeaway

Cycle 0005's three axes (24-byte u32-link residency, size-aware window floor, pairwise conservative update) combine cleanly. Keeping v001's odd-count-corrected `reset()` did not block the hit-rate gain, so that reset difference is not what was holding the 24-byte line back on hit-rate. It was simply missing both levers. The `inline(always)` test answers the open question: conservative update's latency penalty was an out-of-line-call artifact, not inherent cost. Get p99 is the only part of latency left to watch.
