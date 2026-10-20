# Variant v001 Insight

## Helped

- per-op-time: capping the ghost at `(main_size / 2).max(1)` instead of `main_size` halves the linear `ghost.iter().any(...)` membership scan that dominates the `put` path, cutting both put p50 and p99 (get is untouched since it never scans the ghost).
- hit-rate: the shorter ghost window did not just preserve reuse detection — it improved hit-rate over the parent's full-length ghost, so the 50% band still spans workload-a's relevant reuse distances while shedding stale, long-evicted keys that the 100% window kept.

## Hurt

- (none) metadata-size is unchanged — only the integer threshold expression in `evict_small()` moved, not the `Entry`/`S3Fifo` struct layout.

## Takeaway

Halving the ghost cap to 50% of main strictly dominated the parent's full-length ghost on this workload: it banked the put-latency win the cycle-0003 ghost-shrink exposed *and* nudged hit-rate up, proving the 100% ghost was paying scan cost for reuse-distance coverage it never used. The frontier knee sits at or below 50%, where the recoverable near-reuse hits live without the over-long-window scan penalty.
