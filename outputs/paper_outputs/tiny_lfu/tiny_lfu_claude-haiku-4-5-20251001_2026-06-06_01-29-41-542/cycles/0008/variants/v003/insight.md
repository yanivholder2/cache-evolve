# Variant v003 Insight

## Hurt

- hit-rate: Random sampling of victim candidates lost 1% hit-rate compared to exhaustive full-cache scan, suggesting the deterministic head-to-tail ordering exploits the LRU property more effectively than random candidate selection.

## Takeaway

Replacing TinyLFU's full-cache victim scan with random sampling (K candidates drawn uniformly from cache) degraded hit-rate. The hypothesis claimed the deterministic head-to-tail scan introduced bias, but the evidence shows it was actually integral: by naturally prioritizing older items near the head and reducing the age-computation cost for frequent eviction targets, the linear scan discovers better eviction equilibria than random sampling. Randomization increased the likelihood of sampling young, hot items that aren't optimal victims—undoing the synergy between LRU ordering and the age-frequency scoring formula.
