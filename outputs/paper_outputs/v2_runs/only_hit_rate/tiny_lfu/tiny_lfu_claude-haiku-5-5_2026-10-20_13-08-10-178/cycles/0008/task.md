# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0008

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried a reduction in sketch row count, a conservative-update rule that raises only the counters at the row minimum, and periodic global aging of sketch counters. The only Winner, v002, was the conservative-update change, a modest hit-rate gain that came from tightening over-estimates rather than from any new memory.

## Patterns

- Both sketch-side changes that touched the estimator's error were the main levers this cycle. The Winner reduced collision over-counting by changing the update rule, which extends the collision-error lever first seen in Cycle 0004's width gain.
- Removing estimator depth hurt. The min-across-rows estimate depends on enough independent rows to suppress collisions, so the cohort's depth reduction undid the kind of gain that width provided.
- Aging the sketch on a fixed access-count trigger hurt. Discarding accumulated frequency history at this capacity weakened the admission comparison, so the sketch needs its history intact on this trace.
- The Winner's per-operation cost is unmeasured. Its extra minimum-row pass on every access is a plausible cost, and the fitness vector does not report timing to confirm it is acceptable.
- All results come from one trace at one cache size, so the Winner's gain is provisional.

## Open questions

- Does the conservative-update gain hold on other workloads and cache sizes, and what does its extra pass cost per operation?
- Is there an aging trigger, not tied only to access count, that keeps the admission comparison intact while still shedding stale frequency?
- Does conservative update stack cleanly with the Cycle 0004 width change, or do the two gains overlap?

## Continuation

Next cycle, build on the conservative-update rule from v002: test it on additional workloads and sizes, and pair it with a timing signal before stacking more estimator changes. Drop reductions in sketch depth, and avoid global halving on a fixed access-count trigger; any future aging idea should use a different trigger and be tested against the Winner.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried a conservative-update refinement on top of the wider sketch, a larger protected share within the main region, and a doorkeeper that gates first-touch keys out of the frequency sketch. The only Winner, v002, was the protected-share increase, a small hit-rate gain; the doorkeeper variant lost hit-rate and the conservative-update variant moved only slightly.

## Patterns

- Protected-segment sizing is the lever that moved in the Winner. It extends the Cycle 0003 finding that main-region recency structure matters, and it is the second time the protected/probation split has been the source of a gain after Cycle 0004 found the incumbent ratio was better than a halved cap.
- Estimator refinements are stacking with diminishing returns. The conservative-update rule gained only a little on top of the width change from Cycle 0004, which suggests the two act on overlapping collision error rather than independent sources.
- Filtering one-touch keys out of the sketch's update path hurt. The correction applied at estimate time does not fully replace the withheld increments, so the sketch's frequency history is still doing work that a gating shortcut loses.
- The Winner's attribution is not fully confirmed: its source change could not be inspected from the variant record, so the gain is credited to the segment split on the basis of the hypothesis and the fitness delta alone.
- Every result comes from one trace at one cache size, so all gains here are provisional.

## Open questions

- Does the larger protected share hold on other workloads and cache sizes, or does it trade away probation room that matters elsewhere?
- Does the conservative-update gain justify its per-operation cost once the width change is in place, and is the stacked benefit distinguishable from the width gain alone?
- Is there a doorkeeper-style estimate correction that matches the withheld sketch increments closely enough to be worth retrying?

## Continuation

Next cycle, confirm the protected-share gain on a second workload or cache size before building on it, and keep the width and conservative-update estimator changes as the baseline rather than stacking new estimator tweaks on the same over-counting error. Drop the doorkeeper-as-update-filter family in its current form.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried a deeper sketch (one more hash row), a larger protected share within the main region, and a promotion rule that gates probation-to-protected moves on the sketch estimate. Only one gain was reported, a slight hit-rate rise from the added sketch row (v002); the protected-share increase and the gated promotion both lost ground against the parent. The Winners list names v001, but v001's record reports a slight hit-rate loss, so the Winner designation needs checking before anything is built on it.

## Patterns

- Adding estimator depth helped where removing it hurt in Cycle 0005. Extra rows give the min-across-rows estimate more independent samples, which again points at collision error as the lever in the admission comparison.
- The protected share has a peak rather than a monotone trend. Cycle 0006 credited a larger protected share with a gain; pushing it further here reversed that, which suggests the useful range is close to the current setting and that probation room matters once the share gets large.
- Gating promotion on the sketch estimate hurt. Frequency counts lag behind recency, so the gate kept some entries in probation that were about to earn a second hit. Promotion on every hit appears to be the better rule on this trace.
- Depth's memory cost is unmeasured. The added row grows sketch counter storage in proportion, and no variant reports that footprint or any timing, so the gain cannot yet be weighed against its cost.
- Every result comes from one trace at one cache size, so all differences here are provisional.

## Open questions

- Is the Winners designation for v001 correct? Its record shows a hit-rate loss, while the only reported gain is v002's depth change.
- Does the depth gain hold on other workloads and sizes, and where does adding rows stop paying for the extra counter memory?
- Where is the protected-share optimum on other traces, and does it sit near the current ratio or somewhere between it and the reversed setting?

## Continuation

Next cycle, resolve the Winner attribution first: confirm whether v001 or v002 is the real advance before building on either. If the depth gain is kept, test it on a second workload or size and measure its counter memory before stacking further sketch changes. Hold the protected share at or near the incumbent ratio rather than pushing it higher, and drop the sketch-gated promotion rule.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
