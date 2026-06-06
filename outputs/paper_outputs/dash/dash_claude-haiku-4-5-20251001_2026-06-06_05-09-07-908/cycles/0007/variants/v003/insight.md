# Variant v003 Insight

## Helped

- hit-rate: More permissive frequency-gating with divisor 2 (threshold=12) allows additional items to promote to the probing tier, reducing false negatives where hot items were filtered by stricter gates.
- per-op-time: Lower promotion threshold places frequently accessed items into the probing tier sooner, shortening the access path and reducing both p50 and p99 latencies.

## Hurt

(none)

## Takeaway

Divisor 2 successfully extends the permissiveness frontier beyond divisor 3, demonstrating that a more lenient frequency gate better matches the candidacy distribution of the baseline workload. By lowering the barrier to promotion, the variant captures items that deserve higher-tier placement, improving both cache effectiveness and access latency without trade-off.
