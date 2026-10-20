# Variant v003 Insight

## Helped

- hit-rate: Splitting each bucket into probationary/protected segments gives twice-hit keys a reservation that one-hit scan bursts can't evict, lifting hit-rate above the front-admission ceiling the baseline ClassicLRU couldn't cross.

## Hurt

- per-op-time: Both get and put got slower — a hit now does a segment-boundary computation plus a single-element `rotate_left`, and promotion/demotion bookkeeping, where the baseline LRU did a cheaper move-to-back; the promotion rotation also pushes put's tail latency up.

## Takeaway

Encoding SLRU as a `protected_len` boundary over one Vec buys a real hit-rate gain from segment separation while keeping per-item metadata flat (the boundary lives off-`Item` on the bucket struct). The cost is paid in latency: every access now rotates and rebalances the two segments, trading operation speed for eviction quality.
