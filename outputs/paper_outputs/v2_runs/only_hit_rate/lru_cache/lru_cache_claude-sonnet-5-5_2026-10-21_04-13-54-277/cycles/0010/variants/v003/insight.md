# Variant v003 Insight

## Helped

- hit-rate: Replacing strict LRU with a probation/protected SLRU plus a count-min sketch (promotion needs estimate >= 2, admission needs candidate > victim) keeps one-touch keys out of the protected segment and lets scan or one-hit traffic be evicted without displacing frequently reused keys.

## Hurt

- metadata-size: Each node gains a segment tag (`seg`), and the cache adds a 4-row u8 sketch sized to the next power of two of capacity, plus two lists instead of one. This is likely heavier than the baseline's 48-byte node and 112-byte struct, though the eval did not report it.
- per-op-time: Every miss and non-resident put now hashes the key through 4 sketch rows, and `touch()` does an extra estimate on probation hits. The periodic full-table halving also adds work. The eval did not report this dimension, so the cost is unmeasured.

## Takeaway

Frequency-gated promotion and admission is an effective way to beat plain LRU on hit-rate for this trace. It buys that gain with extra per-node and per-cache metadata and extra hashing on the hot path. The vector only covers hit-rate, so whether the trade is worth it is unverified until metadata-size and per-op-time are measured.
