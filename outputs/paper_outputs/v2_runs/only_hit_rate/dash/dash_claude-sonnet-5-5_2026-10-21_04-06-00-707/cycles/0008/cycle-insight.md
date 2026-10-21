# Cycle 0008 Insight

## Cohort outcome

Three refinements of the reuse-based eviction line were tried: a ghost table that restores a counter for recently evicted keys (v001), a non-mutating victim scan with periodic global counter halving (v002), and a second "protected" bit giving probation/protected tiers on the one-bit parent (v003). v001 won, but only by a very small hit-rate gain. v003 gained modestly over its parent, and v002 fell below its parent.

## Patterns

- Gains are small again. The reuse signal, the admission filter and the counter already capture most of what returning or reused keys offer on workload-a. Extra history (ghost table) and extra tiers (protected bit) add little.
- Replacing scan-time counter decrement with global timer-driven halving (v002) hurt. Local, scan-time aging is a useful idleness signal, so aging should stay tied to where the evictor looks.
- v001 and v003 both add state or scan work (ghost writes and lookups, multi-pass scan) whose cost is unmeasured. v001's near-noise gain is probably a poor trade for that.
- Evidence is still one trace and one size, with only hit-rate reported and no overlay sources readable, so mechanisms and costs are inferred.

## Open questions

- Is v001's Winner label meaningful, or noise-level selection as in cycles 4, 5 and 7?
- Does the SLRU-style two-tier idea (v003) stack with the counter incumbent, or does it overlap with it?
- Is any hit-rate headroom left on workload-a, or do other workloads and cache sizes need to be the test?

## Continuation

Treat v001 as a marginal incumbent at best. Drop global-timer aging (v002) and further history or ghost-state ideas. Eviction-state refinement (counters, tiers, scan order, aging) is saturated on workload-a. Next cycle should validate the incumbent on other workloads and cache sizes, and add per-op-time and metadata-size to the fitness vector so small wins can be judged on cost. If it explores new levers, pick ones outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for other access patterns.
