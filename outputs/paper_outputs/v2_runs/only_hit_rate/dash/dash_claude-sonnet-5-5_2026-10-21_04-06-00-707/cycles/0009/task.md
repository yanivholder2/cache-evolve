# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0009

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

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


### Cycle 0007

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


### Cycle 0008

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
