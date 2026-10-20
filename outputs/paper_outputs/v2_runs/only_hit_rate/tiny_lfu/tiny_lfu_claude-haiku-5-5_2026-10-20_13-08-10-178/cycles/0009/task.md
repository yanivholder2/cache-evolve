# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0009

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tried three changes to how probation victims are chosen or credited: a tie-breaking rule change, a two-candidate victim scan, and a ghost-history frequency bonus. The only Winner, v002, compared the two LRU-tail probation entries and evicted the one with the lower sketch estimate; its hit-rate gain is modest.

## Patterns

- The victim-selection step is where this cohort's signal sits. The Winner changed which probation entry leaves, while the tie rule and ghost bonus only nudged decisions at the margin.
- Tie handling matters. Admitting equal-estimate newcomers against probation victims hurt, because ties were previously broken by queue position and that position was protecting older cold entries the sketch could not separate.
- Ghost-history credit is marginal. The +1 bonus for recently evicted keys barely moved hit-rate, and its ghost ring and per-operation work are unmeasured costs, so it does not earn a place as-is.
- The Winner's attribution rests on the hypothesis and the fitness delta, since its source overlay could not be inspected. Treat the victim rule as the likely cause until confirmed.
- Everything comes from one trace at one cache size, so all differences here are provisional.

## Open questions

- Does the colder-of-two-tail rule hold on other workloads and cache sizes, and does a wider scan window beat a two-entry comparison?
- The victim comparison depends on sketch estimates, so does the Winner's gain depend on the width and conservative-update estimator changes from earlier cycles?
- Is the Winner's gain from the victim rule itself, or from a side effect of how the scan changes queue order?

## Continuation

Next cycle, confirm the probation-victim-by-estimate rule on a second workload or cache size before building on it, and test whether a slightly wider tail scan adds anything. Drop the tie-rule change (`>=` against probation victims), and do not pursue the ghost-history bonus in its current form unless a more selective signal is paired with measured metadata and per-operation cost.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
