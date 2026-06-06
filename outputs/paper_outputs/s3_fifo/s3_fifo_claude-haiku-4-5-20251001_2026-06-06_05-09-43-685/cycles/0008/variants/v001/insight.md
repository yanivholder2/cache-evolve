# Variant v001 Insight

## Helped

- **per-op-time (put)**: Filtering the ghost queue to admit only freq > 0 items dramatically reduces ghost queue size and search overhead. The ghost hit detection loop now iterates over a much smaller, higher-fidelity candidate pool, improving put p50 by 344ns (63% faster than parent). This filtering happens at eviction time (infrequent), leaving the hot path unpolluted.

## Hurt

- **hit-rate**: The selective ghost-queue filtering is overly restrictive. By excluding freq=0 items (which never reached the promotion threshold in the admission queue), the variant eliminates re-admission opportunities for items that may be valuable on second access. Hit-rate dropped by 0.61%, indicating that pure signal fidelity cannot fully substitute for diversity in the candidate pool.

- **per-op-time (get)**: Get latency slightly degraded (p50: 131ns → 136ns, +5ns; p99: 154ns → 168ns, +14ns). Despite the hypothesis's claim of zero hot-path overhead, the structural change from `VecDeque<K>` to `VecDeque<(K, i8)>` plus more aggressive eviction due to stricter re-admission criteria (boosting freq by only +2 instead of starting fresh) introduces subtle contention or cache effects that offset the ghost-queue filtering benefit.

## Takeaway

The variant achieved a dramatic latency improvement for put operations (3.7× faster) by concentrating the ghost queue on high-frequency candidates and eliminating freq=0 noise. However, this signal-purity strategy proved too aggressive—losing 0.61% hit-rate because genuinely warm (but initially freq=0) items were excluded from re-admission opportunities. The trade-off reveals that cache eviction benefits from retaining some diversity in the candidate pool, even at the cost of ghost-queue noise; the ghost queue's value lies partly in its second-chance mechanism for items the small queue rejected, not just in tracking proven access patterns.
