# Variant v002 Insight

## Helped

*None.*

## Hurt

*None.*

## Takeaway

Switching from ClassicLRU to LFU eviction policy did not improve hit rate on this LIRS-style workload. While LFU was correctly implemented across all cache segments and should theoretically protect high-frequency items, the workload's access pattern favors recency over frequency — the very strength of LRU. This suggests that on workloads where temporal locality dominates (recent items are more likely to be reused soon), a recency-based policy is a better predictor of future reuse than access count alone.
