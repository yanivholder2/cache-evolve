# Variant v002 Insight

## Helped

- hit-rate: `CountMinSketch::increment` now raises only the counters that sit at the row minimum, so keys that collide in the sketch stop inflating counters they do not own. `put()` then compares `incoming_freq` and `victim_freq` on less collision noise, which is what moved the admission decisions on workload-a at size 2048.

## Hurt

## Takeaway

The change is confined to the sketch's update rule: the counter table shape is untouched, so the gain comes from the conservative-update policy alone, not from extra memory. The effect is modest, which fits a change that only tightens over-estimates rather than altering admission logic. Per-op-time was not reported in this variant's fitness vector, so the cost of the added row-minimum pass on every `get` and `put` is still unmeasured.
