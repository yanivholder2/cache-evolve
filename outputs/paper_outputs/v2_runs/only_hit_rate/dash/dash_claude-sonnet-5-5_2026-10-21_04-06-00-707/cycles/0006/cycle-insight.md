# Cycle 0006 Insight

## Cohort outcome

Three structurally different levers were tried: a reference-bit victim choice that evicts the first unreferenced slot (v001), power-of-two-choices bucket placement (v002), and a count-min frequency admission gate (v003). Only v001 won, with a clear hit-rate gain on workload-a. This is the first real movement since the cycle 3 admission filter. v002 was flat to marginally lower, and v003 gained only marginally.

## Patterns

- The headroom was on the eviction side. Cycles 1-5 worked on placement and admission and plateaued. A real reuse signal (hit since insertion) used for victim choice broke the plateau.
- Bucket-structure changes (v002) again did not matter, which matches the earlier finding that placement is saturated.
- A frequency-comparison admission gate (v003) added little, so admission is also near saturation. It overlaps with the incumbent's way of screening one-hit-wonders.
- v001 needs no extra metadata in principle, since the flag lives in spare bits. v003's sketch and v002's second probe add cost that is unmeasured.
- Only hit-rate was reported again, on a single trace, and the authors could not read the overlay sources. The size of v001's gain and all cost claims are inferred.

## Open questions

- Does v001's gain hold on other workloads and cache sizes, or is it specific to workload-a?
- Does a richer reuse signal do better than one bit, such as a multi-bit counter, CLOCK-style second chance or hand position? Does it stack with the admission filter?
- What is the CPU cost of the extra bucket and stash scan on eviction?
- Earlier cycles' victim-selection edits may not have been on the live path. Why did this one take effect?

## Continuation

Make v001 the new incumbent and branch from it. Pursue eviction using reuse information: multi-bit or aging reference state, CLOCK-style sweeps, and scan-resistance, while keeping the existing admission filter. Validate it on other workloads and cache sizes. Drop power-of-two placement (v002) and count-min or TinyLFU admission (v003), and do not add more admission machinery. Add per-op-time and metadata-size to the fitness vector so the eviction scan cost can be judged.
