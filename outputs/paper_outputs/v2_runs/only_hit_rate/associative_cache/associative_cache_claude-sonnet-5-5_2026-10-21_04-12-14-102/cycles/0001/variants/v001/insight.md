# Variant v001 Insight

## Helped

- hit-rate: Going from 128 narrow buckets to 32 wider ones (same total capacity) lowers hash-collision conflict misses a little, so hit rate moved up slightly.

## Hurt

- per-op-time: Not measured for this variant, so there is no evidence either way. The hypothesis predicted slower `get` because each bucket is a 64-element Vec with a longer linear key scan and longer remove/push shifts. That cost is untested.
- metadata-size: Not measured. The change only alters the bucket split, so the struct layout probably didn't change.

## Takeaway

Widening associativity from 16 to 64 ways bought only a marginal hit-rate gain on this LIRS trace. Hash imbalance across sets is therefore a minor source of misses here, and most misses are capacity or policy misses that a different eviction policy would address. The hypothesis's expected per-op-time cost was never checked. Without the other two dimensions, the small hit-rate gain can't be judged against that cost.
