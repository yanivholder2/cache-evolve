# Variant v003 Insight

## Hurt

- **per-op-time**: Switching to Swap eviction policy significantly increased latency across all operation types and percentiles; get p50 nearly tripled (69ns→200ns) and put p50 more than doubled (143ns→300ns), indicating Swap's constant-factor overhead dominates practical performance despite its O(1) asymptotic advantage over ClassicLRU's O(n) removal.

## Takeaway

Swap's theoretical constant-time advantage over ClassicLRU did not translate to practical benefits on this workload. Hit-rate remained flat at 0.6611, while operation latency degraded substantially across all measurements. This suggests that the implementation overhead of performing multiple swaps per access, combined with the cost of cache coherency and memory operations, outweighs the asymptotic savings. For this cache geometry and workload, ClassicLRU's recency positioning—despite requiring occasional item removal and reinsertion—is faster in wall-clock time than Swap's bubble-up mechanism.
