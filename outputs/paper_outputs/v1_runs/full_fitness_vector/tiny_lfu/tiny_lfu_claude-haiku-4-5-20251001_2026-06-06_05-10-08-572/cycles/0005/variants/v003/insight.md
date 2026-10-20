# Variant v003 Insight

## Helped

- hit-rate: Median-frequency victim filtering protected above-median-frequency items from premature LRU-based eviction, preventing the cache from displacing genuinely hot items.
- per-op-time (get p50): Slight latency reduction on median-case get operations.

## Hurt

- per-op-time (get p99): Tail latency degraded as the cost of median computation and victim scanning introduces variability in the eviction path.
- per-op-time (put p50): Eviction now requires computing median frequency and potentially scanning the victim list, increasing baseline put latency.
- per-op-time (put p99): Catastrophic regression when the victim search must scan multiple below-median-frequency candidates; worst-case put latency increased 364×.

## Takeaway

Median-frequency victim filtering achieved only a marginal hit-rate gain (0.068% relative improvement) at the cost of severe latency degradation, particularly in put tail latency. The approach of computing the median on every eviction and scanning the linked list for below-median victims introduces too much overhead relative to the modest benefit. Protecting high-frequency items from eviction is a valid insight, but the implementation mechanism—full median recomputation with linear victim search—is too expensive; any future attempt at this lever needs a cheaper detection or decision model.
