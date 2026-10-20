# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0005

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort tried two layout-only changes to the frequency sketch's storage (narrower counters, and a flattened row-major counter array) and one structural change adding a small recency window ahead of the main LRU with admission applied at window exit. Both Winners, v001 and v003, were layout changes that left hit-rate flat; the window variant, v002, lost a little hit-rate and did not advance.

## Patterns

- Layout-only changes to the sketch were the only ones that held hit-rate. Both Winners preserve the logical counter mapping, so their hit-rate is unchanged by construction rather than by a measured improvement.
- Changing when and where admission sees a new key cost hit-rate on this trace; the window's benefit for one-touch bursts was not visible.
- None of the cohort recorded a hit-rate gain. Every benefit claimed by the Winners (memory footprint, per-op cost) sits on dimensions the fitness vector does not measure, so the Winners advance by not regressing.
- The unmeasured-dimension problem carried over from Cycle 0001 is unresolved: layout work keeps producing Winners whose payoff cannot be confirmed.

## Open questions

- Would a memory or per-operation timing dimension in the fitness vector confirm the layout gains, and do they justify keeping both Winners?
- Does the narrower counter type ever create admission ties on traces with very hot keys, where saturation could matter?
- Is there a window or admission structure that helps one-touch bursts without giving up the sketch signal at this capacity?

## Continuation

Next cycle, keep to changes that preserve sketch counts and the admission rule, and prefer layout or allocation work only if the fitness vector gains a metadata or timing signal that can confirm it. Drop the recency-window-in-front-of-LRU structure at this capacity, and do not stack further layout changes until the Winners' gains are measured.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort tried two sketch-layout changes (narrower counters, and hoisting per-key row indices into a shared buffer) and one change to the main region's internal ordering, splitting its LRU into probation and protected segments. The only Winner, v002, was the main-region reordering, the one change that showed a hit-rate movement; the two layout-only variants preserved behaviour but advanced on no measured dimension.

## Patterns

- The only hit-rate gain came from changing recency ordering inside the main region, with the sketch and admission path left alone. That is a different lever from the sketch-side and window-side changes of Cycles 0001 and 0002.
- Layout-only changes to the sketch again preserved hit-rate by construction, and again their claimed benefits (footprint, per-op time) sit on dimensions the fitness vector does not measure. This is the same unmeasured-payoff pattern seen in Cycle 0002.
- The Winner's signal rests on a single trace at a single cache size, so its gain is provisional rather than established.

## Open questions

- Does the probation/protected gain hold on other workloads and cache sizes, or is it specific to this trace?
- Would a timing or memory dimension in the fitness vector confirm the hashing-reuse and counter-narrowing gains, and justify keeping them?
- Does the sketch's saturation point ever bind on very hot keys, or does the admission comparison stay insensitive across workloads?

## Continuation

Next cycle, build on the main-region recency structure from v002: test it on additional workloads and sizes before stacking further changes on it, and avoid re-proposing the window-in-front-of-LRU idea. Drop further sketch-layout-only changes until the fitness vector gains a metadata or timing signal that can confirm them.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried a sketch-width change, a resizing of the protected segment within the main region, and a change to where demoted protected entries land in probation. The only Winner, v002, was the sketch-width change, the first sketch-side change in recent cycles to show a hit-rate gain; the two main-region variants both lost hit-rate against the parent.

## Patterns

- Widening the sketch reduced over-counting from colliding keys and produced the only hit-rate gain in the cohort. This contrasts with Cycle 0001, where sketch-side changes lost hit-rate; the difference is that this change addressed collision error rather than decay or key mapping.
- Both main-region variants moved away from the incumbent and both lost hit-rate. The current probation/protected split and the demotion placement at the back of probation both look like the better settings for this trace.
- The Winner's gain is small, and its predicted metadata cost (sketch size scaling with width) was not measured. Its advancement rests on hit-rate alone, the same unmeasured-cost pattern seen in Cycles 0002 and 0003.
- All three variants were evaluated on a single trace at a single cache size, so every result here is provisional.

## Open questions

- Does the width gain persist on other workloads and cache sizes, and is it worth its metadata cost once that cost is measured?
- Is there a point where further widening stops helping, i.e. where collision error is no longer the limiting factor in admission?
- Does the protected/probation ratio near 4/5 hold as the best setting on other traces, or is it specific to workload-a?

## Continuation

Next cycle, build on the sketch-width result by testing it on additional workloads and sizes and measuring its metadata footprint before stacking further sketch changes. Hold the current protected-segment ratio and demotion placement as-is, and drop the halved protected cap and the push-to-front demotion variants.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
