# Variant v003 Insight

## Helped

- **hit-rate**: Replacing the full remove-and-push promotion in `ClassicLRU::get_from_position` with a single-swap LFU scheme causes frequently-accessed items to gradually migrate toward position 0 (the protected head), so the tail eviction candidate is genuinely cold rather than merely the oldest-promoted item — this frequency-aware ordering lifts hit rate on the workload-a trace.

## Hurt

## Takeaway

Composing the 32-byte aligned `Item` layout from cycle 0002 with a single-swap LFU promotion confirms that the hit-rate gap between the previous ClassicLRU (move-to-tail) and full LFU was largely due to eviction-candidate quality, not struct overhead: the cold tail under LFU ordering is a truer eviction target than the LRU head. Per-op-time did not move, suggesting that at the benchmark's bucket sizes the O(n) remove-and-push was already within the measurement floor, and the position-0 early-return guard's write-elimination benefit is not yet visible at this granularity.
