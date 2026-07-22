# Variant v003 Insight

## Helped

_(none)_

## Hurt

- **hit-rate**: Reverting the cold-candidate scan back to LIFO order (newest-cold first via `.rev()`) causes the most recently inserted, never-accessed items to be evicted before they have time to be re-requested, undoing the hit-rate gain the parent earned by scanning LRU-among-cold (oldest-cold first).

## Takeaway

This variant is a deliberate A/B diagnostic: it reverses the single scan-direction change introduced in cycle/0008/v003 to test whether that direction flip was responsible for a persistent put p99 regression. The per-op-time dimension is identical under both scan orders, which conclusively rules out scan direction as the source of that regression — the cost of iterating the cold window in forward versus reverse order is indistinguishable at the p50 and p99 level. The hit-rate drop is an expected side effect of the diagnostic and confirms that LRU-among-cold ordering (forward scan) is the better eviction choice for this workload; it does not represent a newly discovered regression.
