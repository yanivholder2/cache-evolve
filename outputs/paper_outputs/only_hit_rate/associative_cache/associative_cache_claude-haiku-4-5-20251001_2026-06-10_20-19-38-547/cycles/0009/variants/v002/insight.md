# Variant v002 Insight

## Helped

- **hit-rate**: Replacing FIFO eviction with LRU (based on last-access timestamps) in the hot tier allows the cache to preferentially retain recently re-accessed items, better capturing temporal locality within the hot tier and reducing premature eviction of active keys.

## Hurt

(None measured in this evaluation)

## Takeaway

Age-tracked LRU eviction in the hot tier improves hit-rate over simple FIFO, confirming that temporal recency within the hot tier matters for this workload. However, the gain is modest (+0.5%), suggesting the two-tier stash mechanism (20%/5% partition) already captures most temporal patterns effectively; refining eviction order within the hot tier yields incremental returns.
