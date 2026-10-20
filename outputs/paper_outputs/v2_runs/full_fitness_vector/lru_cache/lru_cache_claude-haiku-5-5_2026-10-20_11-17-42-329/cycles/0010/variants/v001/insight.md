# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The new-key branch of `put` now calls `push_front` instead of `push_back`, so a freshly inserted key sits at the probationary eviction end and becomes the next victim on the next miss at capacity. It survives only if a `get` or overwrite sets its reference bit before that miss. Keys that are re-referenced after one or more intervening misses, which the old MRU-end placement kept alive, are now evicted first. That loss on workload-a at size 2048 is the hypothesis's predicted failure mode, and the data points to it.

## Takeaway

Placing new keys at the eviction end removes the probationary segment's role as a reuse-distance buffer: a one-hit key now gets exactly one miss's worth of time to be re-referenced, rather than most of the probationary segment's length. On workload-a, that cost outweighs any scan resistance, so the insertion-end change is a net loss here. The put p99 movement is not attributable to this change, because `push_front` and `push_back` do essentially the same link work and the `get` path, where p99 also moved, is untouched.
