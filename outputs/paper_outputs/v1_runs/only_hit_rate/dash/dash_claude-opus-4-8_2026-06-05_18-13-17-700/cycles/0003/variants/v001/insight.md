# Variant v001 Insight

## Helped

- (none): Pushing `BaselineGeometry::layout` from the parent's `total_buckets * 3 / 4` (24/8, 3:1) to `total_buckets * 13 / 16` (26/6, ≈4.3:1) did not raise hit-rate; the protected normal tier was already large enough that handing it two more buckets bought nothing.

## Hurt

- hit-rate: Reallocating two more buckets from stash to normal nudged hit-rate fractionally below the parent's 3:1 point, so the curve has flattened and just barely turned over by 4.3:1 — the over-shrunk stash starts costing as much admission as the wider protected tier saves.

## Takeaway

The cycle-0001 normal/stash axis stops climbing right around the parent's 3:1 split: moving to ~4.3:1 left hit-rate essentially unchanged (a hair lower), so the peak between 3:1 and the baseline 7:1 is pinned at or just past 3:1, not deeper toward the baseline ratio. Further starving the stash on this skewed LIRS trace is no longer a budget-preserving win.
