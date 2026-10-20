# Cycle 0006 Insight

## Cohort outcome

Three changes were tried: blocking all sketch rows into one cache-line block (v001), choosing the protected-overflow demotion victim by sketch estimate (v002), and replacing pointer links with u32 slab indices (v003). v002 was named Winner, but it is effectively a tie with its parent, within noise. v001 lost hit-rate, and v003 preserved behavior.

## Patterns

- Same split as earlier cycles: changes that weaken frequency-estimate quality (blocked rows with correlated slots) lost hit-rate. Changes that leave estimates and admission untouched (demotion choice, slab links) were neutral.
- The Winner is again a near-tie. Segment internals (protected size in cycle 4, demotion choice here) have now been flat twice, so they do not move hit-rate on workload-a.
- Locality and cost optimizations (v001, v003) show only their downside, or nothing, because the fitness vectors still report hit-rate only. This is the sixth cycle running with metadata-size and per-op-time unmeasured.
- Variant authors again could not always read overlay source, so explanations rest on hypothesis text and fitness vectors.

## Open questions

- Does v003's slab layout shrink metadata or speed up ops? It is behavior-preserving, so it is a free win if it does.
- Could a blocking scheme with better row decorrelation (for example, a different hash per row within the block) keep hit-rate and still gain locality?
- Is workload-a saturated for hit-rate? Six cycles of tweaks have produced only noise-level movement.

## Continuation

Stop generating hit-rate hypotheses until the evaluation reports metadata-size and per-op-time and includes a second trace (shifting popularity or a different cache size). Keep v003's slab links and the earlier single-hash and index-reuse changes as candidate cost wins, and verify them on size and speed first. Drop segment-internal tuning (protected size, demotion choice) and simple sketch blocking. Revisit blocking only with decorrelated rows, and only if per-op-time is measured.
