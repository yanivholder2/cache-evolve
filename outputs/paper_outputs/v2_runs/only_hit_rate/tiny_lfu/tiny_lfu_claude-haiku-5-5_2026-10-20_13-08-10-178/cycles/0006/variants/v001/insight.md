# Variant v001 Insight

## Helped

- hit-rate: Conservative update in `CountMinSketch::increment` raises only the row counters that sit at the key's current minimum, so colliding keys stop inflating counters that are already above the true frequency; on top of the width-8 sketch from the parent, this removes a further slice of the over-counting that the wider rows left behind, and the admission check in `put()` sees slightly tighter frequency estimates.

## Hurt

## Takeaway

Conservative update stacks with sketch width only in a modest way: the hit-rate moves up slightly over the width-8 parent, which is the direction the hypothesis predicted, but the gain is small enough that the two changes appear to act on largely the same over-counting error rather than independent ones. This is a single trace at a single cache size, and the fitness vector reports only hit-rate, so it does not show whether the sketch's extra cost from width 8 is worth keeping alongside this change.
