# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0007

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This was a settle-the-standing-questions cycle: rather than turning a
proven knob further, all three variants tested a tolerated assumption
from the cross-cycle log — flattening the wide sketch's allocation
(v001), conservative-update counting on the narrow sketch (v002), and
buying back row independence with two real hashes (v003). Only the
conservative-update variant produced a genuine hit-rate gain; the two
locality/independence probes resolved their open questions negatively,
landing flat hit-rate at a per-op cost.

## Patterns

- The wide sketch's per-op tax is confirmed *not* a layout artifact:
  flattening `Vec<Vec<u8>>` to one contiguous buffer left per-op time
  worse, not better. The cost is the footprint and the within-row hash
  stride spilling cache lines, identical under either allocation shape —
  so it is a real floor price, not recoverable by re-shaping memory.
- The tolerated "double-hashing dip" is effectively zero on this trace.
  Genuinely decorrelated rows produced the same per-key min estimate as
  the high-bit-derived second hash, so admission and victim choice were
  unchanged — the row correlation was never the accuracy bottleneck, and
  paying for a second hash pass only bought slower ops.
- Signal-sharpening still wins, but the price has moved onto the hot path.
  Conservative update tightened the estimate and lifted hit-rate without
  touching geometry or metadata — yet because the min-scan now precedes
  every write (get and put both record frequency), it taxed per-op time
  broadly rather than only on eviction.

## Open questions

- Conservative update helped on the narrow factor-4 sketch — does it still
  pay on the wider sketches where collision over-count is already lower,
  or does its hot-path tax then outweigh a smaller accuracy gain?
- With layout and hash-independence both ruled out as recovery levers, is
  the only remaining route to cheaper per-op a smaller/sparser sketch that
  trades back some hit-rate?

## Continuation

Two standing questions are now closed: stop probing sketch allocation
shape (the per-op tax is a genuine floor, not a layout artifact) and stop
revisiting row-hash independence (no accuracy gain here, pure speed cost).
Carry conservative update forward as the cycle's one real win, but test it
on the wider-sketch base before adopting — confirm the estimate gain
survives where collisions are rarer, and weigh its broad hot-path cost.
The frontier remains hit-rate-per-unit-per-op; the cheap free-lunch levers
are gone, so favor edits that sharpen the estimate without adding work to
every access.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
