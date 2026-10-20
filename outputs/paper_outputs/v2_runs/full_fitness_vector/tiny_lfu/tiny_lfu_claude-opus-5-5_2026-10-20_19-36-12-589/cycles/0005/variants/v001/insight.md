# Variant v001 Insight

## Helped

- metadata-size: Removing the redundant `depth` field from `CountMinSketch` (now the compile-time `SKETCH_DEPTH`, enforced by `const _: () = assert!(SKETCH_DEPTH == 4)`) shrinks the inline `TinyLfuCache` struct by one `usize`. The `Node` summary and the sketch heap bytes are unchanged because the `u16` storage and `SKETCH_WIDTH_FACTOR = 8` were kept exactly as in the parent, so the headline value doesn't move.

## Hurt

- per-op-time: The fused single-pass, register-held, `get_unchecked` increment did not lower the headline `get` p50. The tail results split both ways: `get` p99 got worse and `put` p99 got better. The algorithm does the same 4 loads and 4 masked stores as before, so the rewrite only removes a second reload pass and bounds checks. Against a u16 sketch that is about 64 KiB and probably cache-resident, that saving is too small to see, and the mixed tail movement looks like the bench noise the parent's insight warned about, not a real regression.

## Takeaway

Porting cycle 0004/v001's fused, unchecked increment onto v003's u16 sketch kept v003's behaviour bit-for-bit: hit rate is identical, which confirms that the two changes really are orthogonal. But it did not carry over the latency gain, so v001's earlier speedup did not come from removing the reload pass and bounds checks, or it only shows up with 32-bit counters. Sketch micro-optimisation is no longer moving per-op-time. Remaining latency is more likely in the per-op `HashMap` lookup, `key_hash`, and linked-list relinking than in the 4-row counter update.
