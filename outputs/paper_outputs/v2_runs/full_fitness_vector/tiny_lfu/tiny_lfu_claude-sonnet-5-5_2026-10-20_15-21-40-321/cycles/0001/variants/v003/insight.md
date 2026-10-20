# Variant v003 Insight

## Helped

- per-op-time: Hashing the key once and deriving the four row indices from that hash removes three SipHash `DefaultHasher` constructions per sketch call. This matters most on a full-cache `put`, which calls `estimate` twice and `increment` once. Both get and put latency improved, and the put tail improved most.
- hit-rate: Conservative update only bumps the minimum counters, so frequency estimates are less inflated by collisions. This gave a small hit-rate gain, even though the u8 saturation cap is a lower ceiling than u32.
- metadata-size: A flat `Vec<u8>` replaces four `Vec<u32>` rows, so the sketch's heap footprint falls by the expected 4x. The per-entry `Node` and struct sizes did not change.

## Hurt

- metadata-size: The dimension's summary did not move, because it tracks the `Node` size and not the sketch heap. The 4x saving shows only in the per-type breakdown, so a selector that reads only the summary will not see it.

## Takeaway

The cost in this sketch was the repeated per-row hashing and the oversized counters, not the Count-Min structure. A single hash with cheap per-row mixing, plus u8 conservative-update counters, gave a smaller and faster sketch with no loss of admission quality. The u8 saturation cap did not hurt on this trace. Whether it holds on workloads with larger frequency gaps is untested.
