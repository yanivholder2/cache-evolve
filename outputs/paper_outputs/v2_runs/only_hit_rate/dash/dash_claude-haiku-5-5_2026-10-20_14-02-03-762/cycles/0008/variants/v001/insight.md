# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Gating promotions from the probing-tier stash on a saturated reuse counter, while leaving the second-hit gate for other stash entries untouched, did not move hit-rate on LIRS workload-a at size 2048 beyond noise. That suggests probing-origin stash entries that reach a hit are already mostly the ones with persistent reuse, or that they contribute too few hits to matter on this trace. This reading is limited to the hypothesis and fitness vectors: the overlay source could not be read in this pass, so I could not confirm the gate was implemented as described.
