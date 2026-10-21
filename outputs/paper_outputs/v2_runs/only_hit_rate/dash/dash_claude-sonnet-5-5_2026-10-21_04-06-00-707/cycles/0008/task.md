# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0008

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three variants were tried: a 4x larger, never-reset second-sighting filter on the cycle 3 incumbent (v001), a two-tier insert position that puts twice-seen keys at MRU and others next to MRU (v002), and a port of the one-bit filter onto the cycle 4 v002 variant (v003). v001 is the Winner, but only by a noise-level hit-rate gain. v002 was flat. v003 came out marginally below its parent.

## Patterns

- Every change landed within noise of its parent. Filter capacity, filter aging and insert position are all saturated on workload-a.
- Once the second-sighting filter screens out one-hit-wonders, the keys that get through are mostly worth keeping. Ranking them within a bucket adds nothing, which matches the cycle 2 finding that placement does not matter.
- The filter's gain did not stack on a different parent (v003). That parent already blocks some low-value inserts, so the two admission mechanisms overlap and are not independent wins.
- v001 bought a negligible gain with 4x the filter memory, so it is probably a bad trade. Memory is still unmeasured.
- Only hit-rate was reported again, and the authors could not read overlay sources. Cost claims and mechanism claims are still inferred.

## Open questions

- Is the Winner label for v001 meaningful, or is it noise-level selection again, as in cycle 4?
- Does the admission family hold up on other workloads or cache sizes, where filter capacity or placement might matter more?
- Is there any untried lever, such as bucket and probing structure or a different admission signal, that moves hit-rate more than noise?

## Continuation

Treat the cycle 3 v002 filter as the incumbent. Admission tuning (capacity, aging, counters, victim exemption) and insert-position tweaks are now exhausted on workload-a, so drop them. Don't stack admission filters on parents that already screen low-value inserts. Next cycle should try a structurally different lever (bucket and probing structure, or victim selection confirmed live). Alternatively it should validate the incumbent on other workloads and cache sizes. Add metadata-size and per-op-time to the fitness vector first, so noise-level winners like v001 can be rejected on cost.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
