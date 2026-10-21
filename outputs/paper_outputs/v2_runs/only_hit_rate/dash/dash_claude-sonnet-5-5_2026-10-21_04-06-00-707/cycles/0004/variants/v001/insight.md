# Variant v001 Insight

## Helped

- None. hit-rate is the only reported dimension and it did not improve over the parent.

## Hurt

- hit-rate: Replacing v002's one-bit "seen once" marker with a repacked 2-bit saturating counter (half the slots, plus periodic halving) lowered hit-rate on workload-a. The likely cause is that, with the same admit-on-second-sighting rule, the change added no filtering power. It also halved the slot count, which raises fingerprint collisions, and the aging resets forget keys that were seen once, so some reusable keys are rejected again. I could not read the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

The hypothesis kept the admit threshold at "seen once before", so the counter could only add collision and aging noise to v002's filter. A frequency counter would pay off only if the admission threshold or the eviction decision actually used the extra count. Memory-neutral repacking that halves the slots gave up filter accuracy without a compensating benefit.
