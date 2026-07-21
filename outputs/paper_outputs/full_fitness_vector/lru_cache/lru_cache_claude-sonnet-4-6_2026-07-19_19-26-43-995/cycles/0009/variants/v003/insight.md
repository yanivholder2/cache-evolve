# Variant v003 Insight

## Helped

- **hit-rate**: Halving `prob_cap` again (capacity/8 → capacity/16) raises the protected quota to 93.75%, shielding even more moderately-frequent Zipfian items from eviction and delivering a further, if modest, hit-rate gain on top of the parent.
- **metadata-size**: `FnvBuildHasher` is a zero-sized type (no fields), so replacing the default `RandomState` — which stores two `u64` seed keys — removes 16 bytes from the `LruCache` struct, shrinking `LruCache<u64,u64>` from 144 to 128 bytes.

## Hurt

- **per-op-time**: The 93.75% protected quota increases the frequency of demotion events triggered on every probationary hit (the protected queue overflows and kicks its LRU node back to probationary more often), adding queue-manipulation work that the FNV hasher's lower hash cost failed to offset, pushing `get` p50 above the 100 ns measurement floor.

## Takeaway

FNV-1a delivered a real structural benefit — a 16-byte struct-size reduction from eliminating SipHash's random seed storage — but its latency saving was insufficient to compensate for the extra demotion churn at the 93.75% protected quota; the quota tightening, not the hasher swap, is the dominant cost driver. The FNV zero-sized-builder trick is worth carrying forward to any variant that uses a HashMap, but further quota increases beyond 87.5% need a different approach to keep per-op-time in check.
