# Variant v001 Insight

## Helped

- hit-rate: Widening `SKETCH_WIDTH_FACTOR` from 4 to 16 quadruples the count-min sketch columns, quartering frequency-counter collisions so admission decisions are less corrupted by aliasing — lifting hit-rate cleanly to v002's level on top of v001's compact node layout.

## Hurt

- per-op-time: The 4x-larger sketch table (heap grew from 16 KB to 64 KB) spills out of cache lines the hot `position_for` double-hash stride used to touch, nudging `get` p50/p99 up slightly; `put` p99 is unchanged-to-better since its cost is dominated by the eviction walk, not sketch reads.

## Takeaway

The two cycle-0004 winners compose additively: the wider sketch geometry and the `u32`-sentinel node encoding modify disjoint structures, so stacking them yields v002's hit-rate at v001's metadata-size 24, confirming the predicted clean stack. The only cost is a small per-op-time tax from the larger sketch's worse cache locality — the hit-rate gain is paid for in memory bandwidth, not metadata footprint.
