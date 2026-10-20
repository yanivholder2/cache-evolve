# Variant v002 Insight

## Helped

(none)

## Hurt

- per-op-time: Collapsing the four inner `Vec<u32>` rows into one flat allocation places each sketch row `width × 4` bytes apart; at the evaluated capacity those strides land in competing cache-set regions, causing the four row accesses in `increment` and `estimate` to thrash L1 rather than hit it — the opposite of the expected indirection reduction.

## Takeaway

Eliminating pointer indirection in `CountMinSketch` by flattening to a single `Vec<u32>` made per-op latency more than double. The row stride in the flat layout (one multiply per access, plus rows separated by tens of kilobytes) creates cache-set conflicts across the four rows on every hot-path call, while the baseline's four independent heap allocations happened to land in more L1-friendly locations. Pointer elimination is not a reliable win when the resulting stride amplifies set-associativity pressure.
