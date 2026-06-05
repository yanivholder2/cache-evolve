# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The hybrid frequency-recency scoring formula (score = frequency × 100 + position_from_head) replaced the two-pass min-frequency selection, creating sub-optimal victim discrimination under aggressive frequency decay. With frequencies constantly decayed to low values every 64 operations, the position term (0-2047) dominates the score and conflicts with LRU tiebreaking, causing worse overall eviction decisions on the workload.

## Takeaway

Hybrid frequency-recency scoring designed to improve eviction decisions actually regressed hit-rate when layered into a cache with multiplicative 0.75× frequency decay. The parent's two-pass approach—find minimum frequency, then select oldest within that tier—specifically targets the low-frequency regime created by decay. The single-score formula blends frequency and position into intermediate decisions that don't match the workload's access patterns, suggesting that for frequency-decayed caches, explicit two-pass logic with LRU fallback outperforms blended scoring.
