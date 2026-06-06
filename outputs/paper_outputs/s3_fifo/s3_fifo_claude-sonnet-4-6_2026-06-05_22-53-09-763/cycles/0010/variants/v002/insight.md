# Variant v002 Insight

## Helped

- hit-rate: Shrinking the small queue from ~5% (`capacity / 20`, 102 slots at 2048) to ~3.6% (`capacity / 28`, 73 slots) transferred ~29 additional resident slots to main, and the cold-admission filter remained effective enough to continue reducing one-time-access pollution at this tighter allocation.

## Hurt

## Takeaway

The monotone hit-rate improvement extends to a ~3.6% small queue: the admission filter still rejects enough single-access traffic at 73 slots that the enlarged main queue retains higher-quality residents. The incremental gain from this step is visibly smaller than the gain v001 achieved by moving from `capacity / 14` to `capacity / 20`, suggesting the benefit curve is beginning to flatten and the true floor is likely within one or two similar-sized denominator steps from here.
