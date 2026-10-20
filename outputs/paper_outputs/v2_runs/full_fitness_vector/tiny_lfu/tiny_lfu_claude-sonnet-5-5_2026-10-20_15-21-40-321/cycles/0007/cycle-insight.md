# Cycle 0007 Insight

## Cohort outcome

Three non-sketch changes were tried on different parents: pre-sizing the map and slab with in-place slot reuse (v001), inlining and branch hints on the `get` fast path (v002), and a segmented probation/protected main LRU on the baseline (v003). All three were kept as Winners. Only v003 showed a plausible real change: a smaller Node and faster get and put, with a slight hit-rate loss. v001 and v002 were flat or worse.

## Patterns

- Allocation and growth are not a bottleneck. Pre-sizing and slot reuse changed nothing measurable, because the cache fills early and steady-state growth cost is negligible.
- Forcing inlining and branch hints on helpers shared by `get` and `put` did not help and may have slowed both. Call overhead, duplicate hashing and branch layout are not where `get` spends its time. The broad slowdown may also be noise.
- Segmenting the main LRU did not raise hit-rate. The sketch admission filter already supplies most of the scan resistance this trace needs, so main-region scan resistance is not the hit-rate limiter.
- v003's speed and size gain came from the compact Node, with segment membership packed into a spare bit, not from the segmentation itself. This again points to memory footprint and locality as the lever that pays.
- Hit-rate was flat or slightly down across the cohort, so the plateau on workload-a holds against a structural mechanism too.
- Two insights were written without a source diff, so some attributions are inferred. Small latency deltas remain unreliable without repeat runs.

## Open questions

- Is v003's speed gain driven by the compact Node alone? If so, can it be reproduced on the stronger incumbents without the segmentation and its hit-rate cost?
- Where does time actually go in `get` and `put`? There is still no profile.
- Is any hit-rate lever left on workload-a in window sizing or the admission decision itself?

## Continuation

Drop pre-sizing, `#[inline(always)]` and branch-hint tweaks, and drop segmenting the main LRU as a hit-rate idea. Pursue footprint and locality: carry the compact 24-byte Node layout onto the best incumbent without the segmentation, and test whether the speed gain survives repeat runs. For hit-rate, look at window sizing or the admission decision, and have each hypothesis name which cost it expects to move.
