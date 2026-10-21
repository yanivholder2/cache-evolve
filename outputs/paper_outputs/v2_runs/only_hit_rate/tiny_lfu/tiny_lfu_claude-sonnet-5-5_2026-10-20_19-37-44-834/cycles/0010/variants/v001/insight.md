# Variant v001 Insight

## Helped

- hit-rate: Doubling the sketch's per-row counter count (H01) means fewer distinct keys share a counter, so frequency estimates used in the admit decision are slightly less inflated. The gain over v002 is marginal, within what could be noise on a single trace and cache size.

## Hurt

- None observed. hit-rate is the only measured dimension, and it did not regress.

## Takeaway

Widening the count-min sketch 2x on top of v002 gives almost nothing on this workload, so collision noise in the sketch is not what limits hit-rate here. The sketch was already wide enough at the 2048-entry cache size. Later cycles should look at the admission or tie-break policy, or at the aging schedule, rather than at more sketch width. Note that I could not inspect the source diff, so this reading rests on the hypothesis text and the fitness vectors.
