# Variant v003 Insight

## Helped

- **per-op-time**: Adding `#[repr(C)]` and an 8-byte `_pad` field brought `Item<u64,u64>` to exactly 32 bytes, packing two items per 64-byte cache line with no cross-boundary splits — the p99 tail latency for both get and put halved compared to the 24-byte parent, confirming that the cache-line straddling every third 24-byte item was the source of the worst-case latency spikes.

## Hurt

- **metadata-size**: The padding field adds 8 bytes back to the per-item footprint, increasing `Item<u64,u64>` from 24 bytes (parent) to 32 bytes, partially reversing the storage gain that removing `timestamp` achieved.

## Takeaway

Cache-line alignment was the real culprit behind v003's p99 regression: padding a 24-byte struct to the next power-of-two size (32) eliminated tail-latency spikes entirely by ensuring no item straddles a cache-line boundary. The trade-off is a partial metadata-size give-back — the struct is now 32 bytes rather than the parent's 24, though still below the 40-byte baseline — demonstrating that alignment padding is a viable way to recover latency from a struct-shrink without fully surrendering the footprint win.
