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
