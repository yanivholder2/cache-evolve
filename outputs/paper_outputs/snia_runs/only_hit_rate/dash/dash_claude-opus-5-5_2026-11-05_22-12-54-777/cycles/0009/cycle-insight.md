# Cycle 0009 Insight

## Cohort outcome

All three variants built on cycle 0008's v002, the contention-gated ghost bypass. Each tried a different way to win back the large-size web_0 loss that v002 caused, without giving up its small-size gains:

- v001 relaxed the gate threshold to admit over counter-1 window minima.
- v002 added the probing neighbour bucket as a second cold-slot placement.
- v003 added a learned per-segment re-hit score as an extra gate branch.

v002 was the Winner, but only because it was the one variant that did no harm. Its gains were noise-level in every cell. In practice the cycle kept the parent and recovered nothing for large-size web_0.

## Patterns

- The single global gate threshold is now used up from both directions. Counter-0-only (c0008) blocks large-size web_0 returners. Counter ≤ 1 (v001) frees them, but it costs more on small-size hm_0, where counter-1 residents are worth protecting. The threshold moves the trade-off between cache sizes. It does not improve both sizes at once.
- More placement options do not help under the same cold-only rule. The neighbour bucket shares the segment and its load, so when the target window is all warm, the neighbour's window is too (v002). The limit is the admission rule, not a lack of nearby slots.
- The learned re-hit signal (v003) did nothing that could be measured. Either the all-warm case rarely triggers it, or "re-hit in target before eviction" is too pessimistic a measure of a returner's value under the demotion cascade. Without counts of how often each gate branch fires, the two cannot be told apart.
- The large-size web_0 gap is real and sits exactly at counter-1 windows. v001 confirmed that all-warm windows at 131072 bottom out mostly at counter 1. The returners worth admitting there compete with counter-1 residents, and on small hm_0 those same counter-1 residents are the ones worth keeping.
- Overlay checks again failed for two of the three variants, so v002's and v003's mechanisms are inferred, not confirmed. Per-op-time and metadata-size are still unmeasured, now for nine cycles. v002's second window read and v003's bitmap both have unknown cost.

## Open questions

- Would a threshold that depends on size or contention keep both sides? For example, ≤ 1 when the target is large or uncontended, and == 0 when it is small or contended, possibly decided from segment occupancy instead of a hard-coded size.
- Does a tie-break that prefers the resident at equal counter keep v001's large-size web_0 gain while removing its small-size hm_0 loss?
- How often does each gate branch actually fire, and how is the v003 score distributed? Instrumentation is needed before any adaptive gate is built on a signal.

## Continuation

Keep cycle 0009 v002 as the parent, or treat 0008 v002 as equivalent, because the gain is noise. Next, try a gate threshold that depends on contention or occupancy: relaxed (≤ 1) only where targets are uncontended, strict (== 0) where they are contended. Separately, try an equal-counter tie-break that favours the resident. Drop placement-spreading (neighbour buckets) and the re-hit-before-eviction feedback score unless they first come with branch-firing instrumentation. Variant insights must confirm the overlay landed, and per-op-time and metadata-size still need to be measured.
