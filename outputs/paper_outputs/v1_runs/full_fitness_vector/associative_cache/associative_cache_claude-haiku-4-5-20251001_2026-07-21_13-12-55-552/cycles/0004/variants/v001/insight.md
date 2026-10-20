# Variant v001 Insight

## Helped

(No dimensions improved)

## Hurt

- **per-op-time**: Switching from position-based (ClassicLRU) to timestamp-based eviction candidate selection forces an O(n) bucket scan with `min_by_key(timestamp)` instead of an O(1) lookup, causing get p99 latency to double (+100ns) and put operations to degrade significantly (+100-200ns). The removal of per-hit bookkeeping does not offset this eviction search overhead, especially under variable bucket occupancy.

## Takeaway

FIFO eviction was successfully implemented (hit-rate unchanged, confirming correct FIFO semantics), but the timestamp-based victim selection strategy is a latency bottleneck. Finding the minimum insertion timestamp across up to 32 items per bucket during eviction is more expensive than ClassicLRU's O(1) position-zero lookup. Removing per-hit bookkeeping from the hit path provides insufficient savings to justify the costlier eviction candidate selection; a correct FIFO implementation needs a faster way to identify the oldest item without scanning the entire bucket.
