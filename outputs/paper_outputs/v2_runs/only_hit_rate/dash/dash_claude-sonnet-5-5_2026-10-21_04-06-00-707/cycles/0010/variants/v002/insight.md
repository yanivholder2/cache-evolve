# Variant v002 Insight

## Helped

- None measured. hit-rate is the only reported dimension, and it did not improve. The simpler eviction path (always evict a victim, no all-hot-bucket check) is the only plausible gain, and no reported dimension captures it. I could not inspect the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Hurt

- hit-rate: Removing the all-hot-bucket drop rule lets an incoming key evict a hot entry when every slot has a nonzero counter. The last-slot-among-ties pick is only a positional proxy for insertion age, so it can evict a useful entry. The result is a small regression against the parent.

## Takeaway

Reverting the drop rule cost a little hit-rate. So the rule did contribute a small amount of protection for fully reused buckets, even though the parent's gain over its own parent looked like noise. The slot-position tie-break did not make up the difference. This is one trace and one size, so the margin is small and could shift on other workloads.
