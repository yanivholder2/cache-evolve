# Variant v001 Insight

## Helped

- metadata-size: Changing `CountMinSketch::counts` from `Vec<u16>` to `Vec<u8>` while keeping depth and width the same halves the sketch's heap footprint exactly, as `heap_bytes()` now reports. This fixed per-cache cost doesn't appear in the headline `Node<u64,u64>` summary, which is unchanged.
- per-op-time: The tails tightened on both ops and `put` p50 fell. This fits a smaller counter table: the four scattered row probes in `increment_hash`/`estimate_hash` miss cache less often. `put` gains the most because a full-cache insert does two sketch walks (incoming increment plus victim estimate), and `get` does only one.
- hit-rate: Saturating at 255 instead of 65535 left admission decisions unchanged on this trace. That supports the hypothesis's reasoning: the strict `incoming_freq > victim_freq` duel is decided by low counts on cold newcomers versus cold CLOCK victims, where the extra counter range never mattered.

## Hurt

- per-op-time: The headline `get` p50 did not move. Halving the table did not speed up the typical hit, so the median `get` is limited by the `HashMap` lookup plus hashing, not by sketch memory traffic. This matches the parent's finding that removing pointer relinking also left the median unchanged.

## Takeaway

`u16` counters were wasted width. At this sketch geometry `u8` saturating counters cost no hit rate, halve sketch memory and trim `put` latency plus both tails, so narrowing counters works where decay did not. Median `get` latency is still stuck, which strengthens the case that the next lever is the map lookup or hashing path, not the sketch. With `u8` counters the next narrowing step would be packed 4-bit counters, which TinyLFU commonly uses. That step needs nibble masking and would test whether the duel tolerates saturation at 15.
