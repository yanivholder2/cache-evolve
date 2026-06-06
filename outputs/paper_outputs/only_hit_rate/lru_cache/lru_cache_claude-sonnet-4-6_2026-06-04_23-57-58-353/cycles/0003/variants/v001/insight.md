# Variant v001 Insight

## Helped

- hit-rate: Ghost admission allowed a small number of keys evicted from the probationary segment to bypass the one-hit filter on re-insertion, placing them directly into protected and avoiding a second eviction cycle.

## Hurt

_(none — no dimension regressed)_

## Takeaway

Ghost admission grafted onto the cycle-1 SLRU baseline produced only a negligible lift in hit-rate, confirming that the ghost set contributes very little marginal value on workload-a. The ghost queue is bounded to `prob_cap` entries (≈20 % of total capacity), which is too narrow a window to capture meaningful cross-residency reuse in this workload — most keys evicted from probationary simply do not return within that window. This isolates the source of cycle-2 v003's earlier gain: it was the SLRU segmentation itself that did the work, not the ghost admission mechanism layered on top.
