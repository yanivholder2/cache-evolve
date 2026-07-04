# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Placing new cold items at the back shifts the cold-first linear scan to always evict the oldest front-cold items first; mid-point insertion previously spread new cold items to the centre, occasionally causing them to be evicted before certain older front items and slightly protecting those front items from premature removal when they were about to be re-accessed.

## Takeaway

Back-append eliminates the O(n/2) shift cost of mid-point insertion, but the only measured fitness dimension — hit-rate — shows a small regression, and per-op-time (the primary target of this change) was never captured in the evaluation. The insertion position is not fully position-independent for hit-rate: mid-point placement subtly interleaved new cold items with older ones in the eviction queue, creating a mild ordering advantage that back-append loses. Until per-op-time is added to the fitness vector, the trade-off between the latency saving and the eviction-order cost remains unresolvable.
