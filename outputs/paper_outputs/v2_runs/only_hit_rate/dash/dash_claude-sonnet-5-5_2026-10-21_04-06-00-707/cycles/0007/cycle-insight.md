# Cycle 0007 Insight

## Cohort outcome

Three refinements of the cycle 6 reuse-bit eviction incumbent were tried: a two-bit aged counter replacing the one-bit flag (v001), a changed victim scan order with second-chance clearing (v002), and letting keys bypass the admission filter when free slots exist (v003). Only v001 won, with a modest hit-rate gain on workload-a. v002 was identical to its parent, and v003 dropped slightly.

## Patterns

- Gains shrank sharply compared with the cycle 6 move. Most of the benefit came from using a reuse signal at all, and finer state (counters, aging) added only a little.
- Scan order and the all-referenced fallback (v002) had no effect. The eviction path is saturated in structure, not just in signal richness.
- Bypassing admission for free space (v003) hurt slightly. The admission filter is not redundant with reuse-bit eviction, because it also keeps cold keys out of empty slots.
- Aging counters on every scanned slot may cost CPU time. This is unmeasured, as is metadata size.
- Evidence still comes from one trace and one size, and authors could not read the overlay sources, so mechanisms and costs are inferred.

## Open questions

- Does the counter gain hold on other workloads and cache sizes, or is it noise-level on workload-a?
- Is the counter's eviction-scan cost worth a modest gain?
- With admission, placement and eviction all near saturation on workload-a, is any hit-rate headroom left on this trace?

## Continuation

Make v001 the incumbent but treat it as a small win. Drop scan-order and second-chance tweaks (v002) and admission-bypass ideas (v003); keep the admission filter unconditional. Further eviction-state refinement is now diminishing, so next cycle should validate on other workloads and cache sizes and add per-op-time and metadata-size to the fitness vector. Otherwise it should try a lever outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for different access patterns.
