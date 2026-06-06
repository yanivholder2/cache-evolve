# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0006

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Acting on Cycle 0002's call to stop probing isolated levers, this cohort
composed proven wins and opened a previously untouched axis: u8 counters
folded onto the fast hasher (v001/H01), node links re-encoded as u32
sentinels instead of `Option<usize>` (v002/H02), and v003's sampled
eviction migrated onto the cheap double-hashing path (v003/H03). All three
landed in the Winners.

## Patterns

- Composition held: the efficiency levers proved additive, not
  competing. Stacking the fast hasher with narrowed counters cost no
  extra per-op time and no hit-rate, confirming the sketch slack and the
  hasher win combine cleanly rather than stacking their downsides.
- The big move was attacking the long-pinned metadata-size axis directly.
  It had been frozen only because every prior variant touched the sketch
  heap and left node layout alone; halving the per-entry node via u32
  sentinel links was a clean, regression-free win — the cheapest axis was
  the one nobody had looked at.
- The recurring p99 weakness flagged in Cycles 0001–0002 was finally
  addressed, not just accepted: moving sampled eviction onto the single
  cheap hash tamed the eviction-path tail (the sample walk now shares one
  fold instead of paying SipHash per candidate), turning a tail cost into
  a net positive.

## Open questions

- With node layout, counter width, and hasher all now optimized together,
  what axis carries the next pool of slack — is residency/admission logic
  the only remaining frontier?
- The correlated-rows hit-rate dip from double-hashing persists as the one
  standing cost; is it worth buying back row independence, or is it
  permanently subsumed by the speedup it enables?

## Continuation

The "compose the proven winners" thesis paid off and the easy
efficiency/layout slack is now largely harvested across all axes. Next
cycle should pivot from cost-cutting to hit-rate: probe residency and
admission logic (sample width tuning, victim selection, the admission
test itself), since the per-op-time and metadata-size axes are now tight
and further squeezing them will likely only trade against hit-rate.


### Cycle 0004

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Following Cycle 0004's call to compose its winners and keep probing
signal quality, all three variants branched off cycle-0004/v001 and each
turned one knob: stacking the wider sketch onto the compact node layout
(v001), deepening the sketch from 4 to 6 rows (v002), and doubling the
victim-sample depth (v003). All three landed in the Winners, each buying
a hit-rate gain at a modest, axis-local cost.

## Patterns

- The predicted composition held cleanly: widening the sketch on top of
  the compact node layout delivered the wider-sketch hit-rate at the
  compact layout's metadata budget, since the two changes touch disjoint
  structures. The "composition is additive" thesis is now confirmed a
  third time.
- Signal quality kept paying, but every lever now has a price. Sketch
  width, sketch depth, and victim-sample depth each lifted hit-rate, yet
  all three nudged per-op-time or sketch heap the wrong way — the larger
  sketch spills cache lines, deeper rows scan 50% more cells, and deeper
  sampling widens the put p99 eviction tail. The free-lunch signal gains
  of earlier cycles are gone.
- Diminishing returns are visible on the residency axis: doubling the
  victim sample bought only a marginal lift, suggesting the sketch
  estimate — not sample depth — is the binding constraint on eviction
  quality for this trace.

## Open questions

- Now that width and depth each help but each cost, is there a combined
  width/depth sweet spot, or do their per-op and heap costs stack faster
  than their accuracy gains?
- Has eviction-quality sampling effectively saturated, given the deeper
  walk's weak payoff against a real tail cost?
- Is the per-op-time tax from the larger sketch (cache spill) recoverable
  through layout/locality work, or is it the new floor price of sharper
  admission?

## Continuation

Signal-sharpening still wins, but it has crossed from free to priced —
next cycle should fold this cycle's confirmed winners (compact layout +
wider sketch) into the working base and look for the width/depth balance
that maximizes hit-rate per unit of per-op cost rather than turning any
single knob further. Treat victim-sample depth as near-saturated: don't
deepen it again unless paired with a sharper estimate. Watch the
sketch's cache-locality tax as the emerging cost frontier.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
