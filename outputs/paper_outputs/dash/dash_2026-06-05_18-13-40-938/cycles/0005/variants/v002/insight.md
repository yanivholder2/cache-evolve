# Variant v002 Insight

## Helped

## Hurt

## Takeaway

The eligibility-delay mechanism did not improve hit-rate compared to its parent (v001). While H02's `promotion_age` tracking successfully gates further promotions for newly promoted items (requiring them to "prove fitness" at their current tier before advancing), this constraint does not resolve the hit-rate stagnation. This suggests that v001's promotion-path MRU problem is rooted not in *eligibility cadence* (how quickly items become candidates for promotion) but in *tier positioning* (where items land upon arrival). Delaying promotion candidacy leaves the fundamental interaction between eviction pressure and MRU landing positions unchanged, indicating the asymmetry requires a different lever—perhaps insertion position or eviction policy—rather than access-cycle gating.
