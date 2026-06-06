# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants were independent, single-lever tweaks to the CountMinSketch
off a shared baseline: frequency aging (v001), single-hash double-hashing (v002),
and narrowing counters from `u32` to `u8` (v003). The two efficiency-focused
sketch reshapings — cheaper hashing and narrower counters — won; the aging variant
lost.

## Patterns

- Winners both attacked sketch *cost* (per-op-time and metadata-size) while
  leaving residency logic and the admission test untouched, and both paid only a
  marginal hit-rate cost — confirming the sketch carries slack that can be traded
  for speed and memory cheaply.
- The loser touched *admission semantics* by decaying the frequency signal; the
  trace appears to have stable, low true frequencies, so aging erased useful mass
  and the byte-width counters never saturated. Signal-preserving changes beat
  signal-altering ones this cycle.
- Tail latency was the recurring soft spot: both an inline full-table sweep
  (v001) and the byte-narrow access pattern (v003) introduced p99 variance even
  when p50 improved.

## Open questions

- Is frequency aging genuinely a dead end here, or just mistuned? The decay
  window was pinned far too tight and the sweep was unamortized.
- How far can counter narrowing go — does 4-bit packing keep the memory win
  without the p99 regression seen at 8-bit?
- Does double-hashing's correlated-rows hit-rate dip compound with other
  sketch changes, or stay marginal?

## Continuation

Pursue the cheap sketch-efficiency levers further: combine the v002 hashing win
with v003 counter narrowing, and push toward 4-bit packing while watching p99.
Before abandoning aging, retry it with a much wider decay interval and an
amortized (non-inline) sweep. Keep favoring changes that preserve the frequency
signal and residency logic over those that alter admission.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Building on Cycle 0001's winning single-hash sketch, this cohort tried three
distinct levers — narrowing counters to `u8` (v001), swapping SipHash for a
multiply-xorshift hasher (v002), and replacing strict-LRU-head eviction with a
sampled-coldest victim walk (v003). All three landed in the Winners, each
buying a small gain on its target axis.

## Patterns

- Every variant traded a tail-latency cost for its primary win: narrowing and
  sampled eviction both regressed p99 (`as u32` mapping, per-eviction sample
  walk) while the cheap hasher cost a marginal hit-rate dip. The p99 soft spot
  flagged in Cycle 0001 persisted across unrelated changes.
- The two efficiency levers (narrowing, fast hashing) again preserved admission
  semantics and paid essentially zero or marginal hit-rate, confirming the
  sketch's slack. The one residency-logic change (sampled eviction) did lift
  hit-rate but only marginally, and at the heaviest tail cost of the cohort.
- Gains are now incremental and axis-local — the easy single-lever wins off the
  baseline appear largely harvested; each remaining lever moves one axis a little
  while nudging another the wrong way.

## Open questions

- Do the three wins compose? Narrowing + fast hashing + sampled eviction were
  each tested against separate parents; their p99 costs may stack rather than
  cancel.
- Can the recurring p99 regressions be amortized away (e.g. precomputed estimates
  on the eviction path) rather than accepted as the price of each win?
- Is sampled eviction's small lift worth its tail cost at this cache depth, or
  does the sample width need tuning to pay off?

## Continuation

Stop probing isolated single levers off the baseline — that vein is thinning.
Next cycle should compose the proven efficiency winners (u8 counters + fast
hasher) into one variant and confirm the gains are additive, then attack the
shared p99 weakness directly. Treat sampled eviction as provisional: retune its
width or drop it unless its hit-rate lift can be bought without the eviction-path
tail cost.


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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
