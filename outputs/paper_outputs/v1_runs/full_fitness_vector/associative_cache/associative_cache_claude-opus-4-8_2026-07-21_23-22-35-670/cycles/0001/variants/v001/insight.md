# Variant v001 Insight

## Helped

- hit-rate: Quartering the bucket count and quadrupling bucket width raised set-associativity, so each wider bucket absorbs more colliding keys and a few conflict misses that the narrow 128×16 baseline dropped now become hits.

## Hurt

- per-op-time: The 4× wider buckets (64 slots instead of 16) lengthen the linear scan within a bucket, inflating both get and put latency across p50 and p99.

## Takeaway

Repartitioning fixed capacity into fewer, wider buckets trades scan cost for collision tolerance: it buys only a marginal hit-rate gain while measurably slowing every operation, because the associative_cache bucket is scanned linearly and width dominates that cost. The conflict-miss headroom at this working-set size is thin enough that quadrupling associativity is a poor exchange.
