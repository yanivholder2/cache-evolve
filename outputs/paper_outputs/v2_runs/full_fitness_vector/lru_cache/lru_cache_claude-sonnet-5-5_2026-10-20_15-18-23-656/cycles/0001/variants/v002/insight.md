# Variant v002 Insight

## Helped

- hit-rate: Splitting the list into probationary and protected segments means one-touch scan keys are evicted from probationary first, so re-referenced keys in protected survive the scans in the lirs workload-a trace.
- per-op-time (get p50): A hit now does one unlink and one relink at the protected MRU. For keys already in protected, that is the same work as baseline's move-to-front, so the median get did not get slower and came out slightly faster.

## Hurt

- metadata-size: The `seg: bool` field on `Node` pushes it past the next alignment boundary, so each node is larger. `LruCache` also grew from the extra head/tail pair and the `protected_len` counter.
- per-op-time (put p99 and tail): Promotion can trigger a demotion from protected to probationary. Evicting from probationary and falling back to protected adds branching and pointer updates to the put path, which shows up mainly in the tail.

## Takeaway

Adding a second segment to the LRU list bought scan resistance, and the hit-rate gain is the reason to keep this variant. The cost was a bigger node and more expensive worst-case puts, because promotion and demotion add list surgery. Segmentation is therefore a hit-rate-for-metadata-and-tail-latency trade, and the median get path was not hurt.
