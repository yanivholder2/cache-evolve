# Variant v002 Insight

## Helped

- hit-rate: Raising the protected share of the main region from roughly 4/5 to roughly 9/10 lets more re-referenced entries stay in protected instead of being demoted to probation, which is the mechanism H02 proposed for the small gain seen on workload-a at size 2048.

## Hurt

## Takeaway

Widening the protected segment is a small positive lever on this one trace and size, and the gain is in the direction H02 predicted, but it is a single-point result and not a broad win. The source diff for the protected-capacity change could not be inspected from this variant's directories, so the attribution to the segment split rests on the hypothesis plus the fitness delta, not on confirmed code. A second trace or cache size is needed before treating the split as the cause.
