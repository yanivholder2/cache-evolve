# Variant v003 Insight

## Helped

- metadata-size: Narrowing the sketch counters from `Vec<Vec<u32>>` to `Vec<Vec<u8>>` (and reporting `size_of::<u8>()` in `heap_bytes()`) shrank the fixed CountMinSketch heap allocation 4×.
- per-op-time: The 4× smaller counter table is more cache-friendly, so `get`/`put` p50 latencies dropped as the sketch rows fit better in CPU cache.

## Hurt

- per-op-time: `get` p99 regressed even as its p50 improved, suggesting the byte-narrow access pattern added some tail variance despite the faster typical case.

## Takeaway

Counter width is pure metadata overhead that admission never needed: `u32` saturating counters carry no more ordering information than `u8` ones for a trace whose true frequencies stay well under 255, so dropping to 8-bit counters cut the sketch's fixed memory 4× with byte-identical hit-rate (0.7362) and faster median ops. This validates counter-narrowing as a low-risk lever and points toward even tighter 4-bit packing next.
