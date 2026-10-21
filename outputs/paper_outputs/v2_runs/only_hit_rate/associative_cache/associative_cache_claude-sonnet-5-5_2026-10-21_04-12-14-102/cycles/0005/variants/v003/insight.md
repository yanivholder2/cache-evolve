# Variant v003 Insight

## Helped

- hit-rate: Splitting each bucket into a 4-slot probation segment and a protected segment (promote on probation hit, demote protected LRU on overflow) keeps one-touch scan entries from displacing re-referenced entries, which plain per-bucket LRU lets happen on the LIRS workload.

## Hurt

- metadata-size: Not measured. The fitness vector has no entry for this dimension, so the hypothesis's claim that positional segments add no metadata is unverified.
- per-op-time: Not measured. The vector has no entry, so the claimed speedup from dropping ghost and counter state is unverified. Promotion and demotion reorder slots on every hit, which could cost time.

## Takeaway

A fixed-size, positional segmented LRU with no ghost, counter, or adaptive state is enough to improve hit-rate over baseline LRU. Adaptive machinery is not needed for that gain. The evaluation produced only hit-rate, so the "lean" half of the hypothesis (same footprint, same op time) remains untested and needs a rerun that emits all three dimensions.
