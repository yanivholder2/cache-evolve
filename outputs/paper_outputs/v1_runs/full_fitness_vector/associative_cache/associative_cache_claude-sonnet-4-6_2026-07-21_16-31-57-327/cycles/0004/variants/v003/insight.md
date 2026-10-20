# Variant v003 Insight

## Helped

- **hit-rate**: The two-tail comparison in `eviction_candidate_index` — selecting the lower-`lfu_counter` of the last two items rather than unconditionally targeting `len-1` — produced a small but real lift by occasionally sparing a penultimate item that had accumulated more hits than a freshly inserted tail neighbour.

## Hurt

- **metadata-size**: The current overlay carries only `eviction_policy.rs` and omits the `item.rs` that the parent included; without it, `Item<u64,u64>` reverts to the larger base layout, erasing the 32-byte `_pad` alignment optimisation that the parent had established.
- **per-op-time**: The put p99 doubled relative to the parent, likely a consequence of the enlarged `Item` struct (40 bytes vs 32) causing swap pairs to straddle cache-line boundaries more often — though at the benchmark's timer resolution this may also be noise.

## Takeaway

The two-tail eviction comparison is a valid micro-improvement to hit rate on its own, but its value is overshadowed here because the overlay did not preserve the parent's 32-byte `Item` layout: losing the `_pad: u64` field inflated per-item metadata and likely degraded cache-line locality during swaps. Composing this eviction change correctly requires carrying the compact `item.rs` forward alongside it.
