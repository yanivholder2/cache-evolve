# Variant v002 Insight

## Helped

- metadata-size: Reordering and packing the small counters or flags in `DashSegment` removed the padding that the parent carried, so the `Dash<u64,u64,DashSegment>` line shrank as H02 targeted. The summary stays at the `Item<u64,u64>` figure because this change deliberately left `Item` alone.

## Hurt

- per-op-time: Both get and put got slower and their tails widened, which is the opposite of what the parent's per-op movement suggested. The likely cause is that the packed fields now need extra shift and mask work on the get and promotion paths, and at this cache size that decode cost outweighs the smaller segment footprint.

## Takeaway

Packing `DashSegment` delivered the footprint reduction it targeted and left eviction order untouched, so the layout change is safe for replacement behaviour. The same change regressed both hot-path operations, which means the segment's size saving is not free: any future attempt at this layout should be judged on per-op time before it is credited with a gain.
