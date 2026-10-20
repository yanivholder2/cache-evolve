# Variant v002 Insight

## Helped

- metadata-size: No gain. The reference bit fit within the existing 40-byte item and 24-byte cache metadata, so the change stayed free in space.

## Hurt

- hit-rate: Replacing move-to-MRU on hit with a reference bit weakened recency ordering. Combined with insertion at ~len/4, second-chance eviction scanning from the LRU end likely spared entries that were hit once, and that removed the scan resistance v003 had.
- per-op-time: Both get and put got slower. The likely cause is bit packing and masking on every access, plus the eviction scan that clears bits and skips entries. I could not inspect the source, so this is inferred from the hypothesis and may be partly noise.

## Takeaway

Stacking a CLOCK reference bit on v003's low-position insertion did not combine the two scan-resistance mechanisms. It lost both hit rate and speed against the parent. The explicit reorder on hit is what makes the len/4 insertion work, because it promotes proven entries. A coarse one-bit second chance cannot replace it, and the bit-packing and scan overhead undid the expected per-op savings.
