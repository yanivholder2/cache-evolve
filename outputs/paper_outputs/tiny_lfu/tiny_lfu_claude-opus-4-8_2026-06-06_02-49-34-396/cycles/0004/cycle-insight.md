# Cycle 0004 Insight

## Cohort outcome

Acting on Cycle 0003's call to pivot toward hit-rate and the
residency/admission frontier, all three variants branched off v003: a
compact u32-sentinel node layout (v001), a 4x-wider count-min sketch
(v002), and a W-TinyLFU recency window (v003). The two signal- and
layout-level changes landed clean wins; the structural window experiment
underperformed at the tested size.

## Patterns

- The biggest hit-rate lever this cycle was *signal quality, not policy*:
  widening the sketch quartered collisions and sharpened the admission
  test and victim choice enough to clear the parent — and it came free,
  since the extra columns enlarge only the sketch heap the hot path
  already strides through with one cached double-hash, leaving p50/p99 and
  the tracked metadata budget untouched.
- Layout slack was still on the table: re-encoding intrusive links as
  u32 sentinels instead of `Option<usize>` halved the per-node footprint
  with bit-identical hit-rate and even slightly better tail latency, since
  `!= NIL` branch tests are cheaper than `match`-on-`Option`. Compact
  integer links strictly dominate for an index-based list.
- The one residency-logic change — a ~1% LRU admission window — failed to
  pay: a window that small can't buffer enough newcomers to accumulate
  frequency before the gate, so the admission benefit never materialized,
  while the per-node region flag and dual-chain bookkeeping cost real
  metadata bytes and eviction-path p99. Touching admission *structure*
  again lost to changes that sharpened the existing signal.

## Open questions

- Does W-TinyLFU's idea have merit at a larger window fraction, or is the
  admission-window pattern simply mismatched to this trace's cold-start
  profile?
- Is there a sketch width beyond 4x that keeps buying hit-rate, or does
  the collision-recovery gain plateau quickly?
- Do the cycle's two wins compose — does the wider sketch's free hit-rate
  gain hold once stacked on the compact node layout?

## Continuation

The "sharpen the existing signal" thesis beat the "add admission
structure" thesis decisively. Next cycle should compose this cycle's two
winners (u32-sentinel layout + wider sketch) onto one variant and confirm
additivity, then keep probing signal-quality levers (sketch width/depth,
counter precision) before reaching for structural admission machinery.
Treat the recency-window approach as provisional: only revisit it with a
substantially larger window fraction, and drop it otherwise — a 1% window
is pure overhead here.
