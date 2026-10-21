# Cycle 0004 Insight

## Cohort outcome

Three stricter-admission tweaks were tried: a shorter ghost combined with the small-queue shrink (v001), a promotion gate one freq level stricter (v002), and a ghost-hit fast path into main only if freq > 0 (v003). Only v001 won, by a marginal gain. v002 tied its parent and v003 was clearly worse.

## Patterns

- Admission-side sizing (small ratio, ghost length) now looks saturated. The shorter ghost and the smaller small queue are not additive, and the combined gain is tiny.
- Making the promotion gate stricter had no measurable effect. Either the gate is not a binding lever on this trace, or the change did not alter behaviour.
- Filtering ghost hits hurt the most. A ghost hit is strong reuse evidence, and with a very short small queue, keys sent back through small are evicted before they can prove themselves again.
- "Stricter admission wins" holds only for ghost length. It fails for the ghost-hit path, and it is untested for promotion.
- Only hit-rate was measured, again. Evidence is still one trace at one cache size, and the variant authors could not inspect the overlay source.

## Open questions

- Did the v002 gate change actually alter which keys get promoted, or was it a no-op?
- Is any remaining admission or capacity lever worth tuning, or is the S3-FIFO structure at its local optimum on this trace?
- Do these small gains hold on a second trace or cache size?
- Can metadata-size and per-op-time be restored?

## Continuation

Stop tuning capacity ratios, ghost length, and promotion thresholds, and keep v001 as the base. Keep the ghost-hit fast path permissive, and consider loosening it only for ghost hits, not for general promotion. Explore structurally different levers, such as main-eviction policy or reinsertion rules, and verify that each diff changes behaviour. Validate on a second trace and cache size, and restore the metadata and per-op-time measurements.
