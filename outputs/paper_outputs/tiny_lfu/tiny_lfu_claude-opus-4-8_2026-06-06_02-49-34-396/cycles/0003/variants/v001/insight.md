# Variant v001 Insight

## Helped

- metadata-size: Narrowing `CountMinSketch.counts` from `Vec<Vec<u32>>` to `Vec<Vec<u8>>` (and `MAX_COUNT` to `u8::MAX`, `heap_bytes` to `size_of::<u8>()`) cut the sketch's heap counter table 4x while the per-entry `Node` overhead stayed fixed.
- per-op-time: The narrower counters added no hashing or branching — `estimate` only adds a cheap `as u32` widening per row — so get/put p50 held at parent levels (get even edged down), confirming the hasher and counter levers are additive rather than competing.

## Hurt

- hit-rate: Unchanged — the trace's true frequencies never approached u8 saturation, so the saturating `if *counter < MAX_COUNT` guard clamps identically to the u32 version and admission decisions in `put` are bit-for-bit the same.

## Takeaway

For this workload the frequency counters carry far more range than they need: dropping them to u8 reclaims 3/4 of the sketch's heap with zero hit-rate cost and no per-op penalty, because counts stay well below saturation and the `as u32` widening keeps the admission comparison unchanged. The hasher (from v002) and counter-width levers compose cleanly — efficiency gains stack without interaction.
