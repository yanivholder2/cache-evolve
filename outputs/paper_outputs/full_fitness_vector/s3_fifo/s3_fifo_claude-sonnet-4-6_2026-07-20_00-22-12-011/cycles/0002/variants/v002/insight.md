# Variant v002 Insight

## Helped

- **per-op-time**: Dropping the O(n) `ghost.retain()` call on ghost hits (replaced with a no-op on the deque, keeping only the O(1) `ghost_set.remove`) collapsed put p99 from 3100 ns to 600 ns and halved put p50; get p99 also tightened, likely from reduced allocator pressure during previously long retain scans.
- **hit-rate**: Marginal improvement — lazy deletion does not alter which keys are considered live ghosts (ghost_set remains authoritative), so promotion logic is unchanged and the tiny gain reflects the same eviction policy applied more cleanly.

## Hurt

_(none — metadata-size is unchanged; no new fields were added)_

## Takeaway

The `ghost.retain()` call was the sole source of O(n) work on the ghost-hit path: it scanned the entire ghost deque to remove one key even though the HashSet already handled membership in O(1). Converting deletion to lazy tombstoning — removing from the set immediately but deferring deque cleanup to the `evict_small` capacity loop — eliminates that scan entirely, cutting tail put latency by 5× with no change to hit rate or struct layout. The deque remains bounded because the capacity guard in `evict_small` drains tombstones one-at-a-time, spreading the cleanup cost across future evictions rather than concentrating it on each ghost hit.
