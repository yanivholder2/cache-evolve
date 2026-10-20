# Cross-cycle insights

## Recurring wins

- No hit-rate gain has been demonstrated in any cycle. Every Winner so far was neutral against its parent, so "Winner" has meant "did not regress."
- The two-hit small-queue promotion bar appears to earn its place. Relaxing it lost hits (0001), and raising it was also neutral, so the current bar is not the bottleneck.
- Cost-side refactors that leave eviction order unchanged are the only category that has been kept (0002, 0003). They remain unverified until they report per-op-time and metadata-size.

## Recurring failures

- Changes that alter ghost admission or ghost coverage have not paid off. Widening the ghost window lost hit-rate (0003) because ghost hits bypass the small queue and displace main-queue residents.
- Single-constant tweaks to already-tuned axes have been inert across cycles: small-queue sizing (0001), ghost routing (0001), main-queue frequency cap (0002), and promotion bar (0001, 0003). Repeating them at this capacity is unlikely to yield signal.
- Overlay and build edits have repeatedly failed to land or been unverifiable: the v003 layout change was absent (0002), and the v002 `freq > 2` edit could not be confirmed (0003). Results from those variants cannot be attributed to the intended change.
- No variant in any cycle has reported metadata-size or per-op-time. Cost claims have never been checked, so no variant can yet be ranked against the baseline.

## Open questions

- Does the v001 resident-key lookup (0003) reduce per-op-time, and at what metadata cost? This is the only pending cost candidate.
- Is the v003 hit-rate drop in 0002 real, or an artifact of the mismatched overlay? It has not been reproduced on a clean build.
- Does the ghost-window loss in 0003 reverse if ghost hits are admitted through the small queue instead of main? This is the one untested ghost-admission variant.
- Are the inert axes inert at other capacities or on other workloads? All evidence is from one trace (lirs workload-a) at capacity 2048.

## Per-policy notes

- S3-FIFO (all cycles so far): the small-queue promotion bar, small-queue sizing, ghost routing, ghost window size, and main-queue frequency cap have each shown no signal or a loss on this trace. Remaining leverage is more likely in how ghost hits are admitted or in per-op cost than in another constant tweak.
- S3-FIFO cost profile: unmeasured in every cycle. Hit-rate is the only dimension reported, so the trade-off surface is still unknown.
