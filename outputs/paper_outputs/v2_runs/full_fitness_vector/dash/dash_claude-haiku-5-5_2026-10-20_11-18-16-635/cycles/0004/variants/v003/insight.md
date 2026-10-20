# Variant v003 Insight

## Helped

- metadata-size: The hypothesis folded the per-item bookkeeping fields of `Item<u64,u64>` into spare key/value bits or a single packed `u8`, removing the alignment padding that held the record at its former size.
- per-op-time: The get and put paths both got faster even though the change was meant to be layout-only; the smaller item plausibly reduces cache-line footprint per access, but the overlay source could not be read here, so this link is inferred rather than confirmed.

## Hurt

## Takeaway

A layout-only repack of `Item<u64,u64>` moved metadata-size in the intended direction and left hit-rate unchanged, which supports the claim that eviction behaviour is independent of record layout. The per-op-time gain is larger than a size reduction alone would obviously explain, so the attribution to this packing change should be confirmed by re-measuring with only the layout change applied.
