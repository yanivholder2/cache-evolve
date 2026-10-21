# Variant v003 Insight

## Helped

- hit-rate: Replacing strict LRU with 1-bit CLOCK plus a TinyLFU-style sketch admission gate improved hit-rate on workload-a at size 2048. The gate drops one-hit-wonder keys whose estimated frequency is below the hand's victim. The referenced bit keeps re-referenced entries resident.

## Hurt

- metadata-size: The fitness vector has no metadata-size dimension for this variant, so it was not measured. The design adds a 4-row 4-bit count-min sketch and a per-entry referenced bit. It drops the LRU list pointers, so the net effect is unknown.
- per-op-time: This dimension is also missing from the vector. Every lookup now updates the sketch, and a full-cache insert scans for a victim and compares estimates. Per-op cost was likely not tracked or the evaluation was incomplete, so no latency claim can be supported.

## Takeaway

Combining CLOCK with a frequency-based admission gate gave a hit-rate gain over strict LRU on this trace, which supports the idea that the two techniques are complementary. The evaluation covers hit-rate only, so the memory and latency costs of the sketch and the hand sweep are unverified. Any further claim about this design needs those dimensions measured.
