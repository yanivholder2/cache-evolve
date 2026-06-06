# Variant v001 Insight

## Helped

- metadata-size: Narrowing `counts: Vec<Vec<u8>>` with `MAX_COUNT: u8` and `size_of::<u8>()` in `heap_bytes()` shrank the CountMinSketch counter table 4x, exactly the sketch-heap reduction the hypothesis targeted.

## Hurt

- per-op-time: Get and put tail latencies (p99) rose — the per-row `c as u32` widening added to `estimate()` plus the smaller table's changed cache footprint pushed the tails up even as the get p50 held flat.

## Takeaway

Dropping the sketch counters from `u32` to `u8` delivers the intended 4x table shrink with zero hit-rate cost on this trace — the 8-bit saturating ceiling never clips a frequency that matters here, so v002's single-hash admission decisions are reproduced identically. The trade is a tail-latency regression in `estimate()`'s per-row `as u32` mapping, but the median per-op cost is unchanged, so the memory win comes essentially free at typical load.
