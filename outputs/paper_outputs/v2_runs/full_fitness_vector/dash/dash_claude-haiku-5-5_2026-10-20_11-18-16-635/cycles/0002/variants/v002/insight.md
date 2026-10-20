# Variant v002 Insight

## Helped

## Hurt

- per-op-time: The per-stash-bucket occupancy counter adds a read and branch to every `promotion_get()` that misses the target and probing buckets, and the counter must be updated on every insert, remove, demotion, and promotion touching a stash slot. Get latency went up instead of down, so on this workload the stash bucket is rarely empty and the fast path almost never skips the scan.
- per-op-time: Put latency regressed on both p50 and p99, which the get-path change cannot explain by itself. The likely source is the counter maintenance on insert and demotion-into-stash, which this diff adds to the put path.

## Takeaway

The zero-occupancy bypass assumed stash buckets are often empty, and this workload shows they are not, so the added bookkeeping costs more than the skipped scans save. Hit rate and metadata size held because the counter lives in the segment struct and eviction and promotion targets were left alone. Any future stash fast path needs evidence that empty stash buckets are common before paying for maintenance on the write path.
