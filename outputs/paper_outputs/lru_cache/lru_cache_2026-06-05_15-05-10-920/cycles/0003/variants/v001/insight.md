# Variant v001 Insight

## Hurt

- hit-rate: Zone-based partitioning with LRU in the cold zone (2/3 capacity) masks frequency signals. Items with moderate access frequency are evicted by LRU before accumulating the frequency >= 2 threshold needed for promotion to the hot zone, losing the frequency discrimination that made the parent's unified approach effective.

## Helped

## Takeaway

Adding zone-based partitioning on top of frequency decay regresses the cache. The parent variant's single unified structure allows all items to compete fairly on frequency alone, which aligns with this workload's access patterns. The two-zone approach enforces a hard partition where the cold zone uses pure LRU, discarding frequency signal and creating an artificial promotion barrier. For workload-a, items do not exhibit a clear hot/cold split; rather, frequency appears to be the dominant eviction signal across the entire cache. The cost of partitioning and enforcing zone separation outweighs any locality benefits from separating temporal (LRU) and frequency-based logic.
