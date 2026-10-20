# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Raising the protected capacity from roughly 9/10 to roughly 19/20 of main-region capacity left hit-rate on workload-a at size 2048 slightly below the parent, so the protected-share gain did not continue past 9/10 on this trace.

## Takeaway

The protected share is a lever with a peak near 9/10 on this trace and size, not a monotone gain. At 19/20 probation is squeezed to a single slot, so new entries have little room to prove themselves before they compete for protected space, which plausibly explains the reversal. The source diff could not be inspected from this variant's directories, so this attribution rests on the hypothesis and the fitness delta, and it needs confirmation on a second trace or cache size before the 9/10 value is treated as a settled optimum.
