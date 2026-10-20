# Variant v003 Insight

## Helped

- per-op-time: `get` p50 finally dropped, after relinking, sketch size and layout changes had all failed to move it. Replacing `fmix64` in `key_hash` with a single `h ^ (h >> 32)` fold removes two dependent 64-bit multiplies and three shift-xors from the path ahead of `indices()`. That lets the four counter loads in `increment_only` issue sooner. This confirms the hypothesis that the median `get` was bound by the serial ALU chain, not by memory.
- hit-rate: The hit rate stayed within the hypothesis's ±0.0005 acceptance band, so the hypothesis is not refuted. Folding the well-mixed high half of the Fx product into the low half gives `h1` enough entropy, and `h2` (the high half, forced odd) keeps the four double-hashed rows independent enough that Count-Min collisions don't skew the admission duel on this trace.
- metadata-size: No change, as expected. The edit only touches hashing, and the sketch geometry, `u8` counters and `Node` layout are byte-for-byte the same as the parent's.

## Hurt

- per-op-time: `get` p99 rose even though the median fell. The fold adds no work, so the likely cause is variance, or a little clustering of row indices for keys whose Fx products differ only in bits the fold cancels. Either way, the fold buys median latency without making the tail any steadier.
- per-op-time: `put` barely improved even though it calls `key_hash` twice (incoming key plus victim). On the full-cache path, `put` cost is set by `HashMap` remove/insert and the CLOCK sweep, not by hash latency.
- hit-rate: The hit rate edged slightly below the parent's. This is inside the noise band, but the direction fits slightly weaker mixing.

## Takeaway

On this cache's per-access path, the cost was in the hash finaliser, not the sketch's memory traffic. A full `fmix64` avalanche is unnecessary when the Fx multiply has already mixed the high bits and one xor-fold can carry them into `h1`. This is the mirror image of cycle 0009's lesson that adding a mix step raised the median. The fold helped `get` p50 but not the `get` tail or `put`, so the next gains for those must come from map operations or the eviction sweep, not from more hash trimming.
