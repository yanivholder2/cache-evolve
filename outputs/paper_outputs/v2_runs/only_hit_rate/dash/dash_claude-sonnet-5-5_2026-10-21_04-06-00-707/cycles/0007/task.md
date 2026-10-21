# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0007

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants refined the cycle 3 second-sighting admission filter: a 2-bit saturating counter with aging (v001), a victim-conditional filter that admits first sightings when the cold-end victim was never hit (v002), and a small periodically-reset doorkeeper bit filter on the eviction path (v003). None improved hit-rate on workload-a. v002 is listed as Winner, but its own insight reports a hit-rate drop below the parent. It looks like the least-bad variant, not a real gain.

## Patterns

- Every refinement of the parent's unconditional one-bit filter lost hit-rate. The parent's design looks like a local optimum, and extra machinery added noise.
- Filter accuracy mattered more than filter sophistication. Smaller or faster-resetting state (halved slots, 256-bit filter wiped at half fill) raised collisions and forgot once-seen keys, so reusable keys were rejected again.
- A single "victim never hit" bit is a poor value signal. Un-hit entries are often still warm, and exempting them from the filter let one-hit-wonders back in.
- Only hit-rate was reported again. The metadata and per-op-time costs of the extra filter state and miss-path work remain unmeasured.
- Authors could not read the overlay sources, so the mechanisms are inferred.

## Open questions

- Is the parent's admission gain robust on other workloads and cache sizes, or specific to workload-a?
- Would a counter or doorkeeper pay off if the threshold or the eviction decision actually used the extra frequency information, with filter size kept constant?
- Why was a variant that lost hit-rate selected as a Winner? Is the winner rule relative to the cohort, or is the fitness vector missing dimensions?

## Continuation

Treat the cycle 3 v002 one-bit unconditional filter as the incumbent and stop tweaking its admission rule, victim exemption and repacked counters included. Next, explore a different lever: bucket and probing structure, or victim selection confirmed to be on a live code path. Alternatively, test the incumbent on other workloads and sizes. Add per-op-time and metadata-size to the fitness vector before any further filter work.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
