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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

A cost-and-refresh cycle: all three variants branched off cycle-0005/v001
(compact layout + wide sketch) and probed whether the working base could
shed work or refresh its signal without losing hit-rate — trimming sketch
depth (v001), shrinking the victim sample (v002), and adding amortized
counter aging (v003). The two trim probes held hit-rate flat and each
banked one axis-local saving; the aging probe confirmed a long-standing
suspicion negatively.

## Patterns

- On the factor-16 sketch, depth and sample-depth are both past
  saturation for *accuracy*: dropping a min-row or cutting the victim
  sample left hit-rate essentially flat, confirming the wide sketch's
  estimate — not row count or walk length — is the binding constraint on
  admission and eviction quality. Their value is now purely cost-side:
  one buys footprint (a quarter of the sketch heap), the other shaves the
  put p99 tail.
- The predicted per-op-time *speedups* did not materialize. Removing one
  counter touch per access and two `estimate` calls per eviction both sat
  below the benchmark noise floor, so the summary per-op metric moved on
  run-to-run variance, sometimes the wrong way. At this width these knobs
  are size/tail levers, not hot-path-speed levers.
- Counter aging is settled as a net loss on this trace. Even tuned wide
  (interval = sketch width, single amortized sweep), halving decayed
  earned counts for a stable hot set faster than it refreshed contrast,
  dropping hit-rate while taxing every `increment`. The base's
  monotonically inflating counters were never the bottleneck here.

## Open questions

- With depth and sample-depth confirmed accuracy-saturated, is footprint
  (drop a row) worth banking permanently, or kept in reserve since it
  doesn't touch hit-rate?
- Is there any remaining lever that sharpens the wide sketch's estimate
  itself — the one thing shown to actually move hit-rate — that hasn't
  already been ruled flat or priced?

## Continuation

Close the aging thread for good: classic counter halving is a confirmed
loss on this stable-hot-set workload — do not revisit it. Treat sketch
depth and victim-sample depth as accuracy-saturated; tune them only for
footprint or tail, never expecting hit-rate or per-op speed. The frontier
is unchanged and narrowing: the only demonstrated hit-rate mover is
estimate sharpness (conservative update from Cycle 0006 remains the live
candidate), and the cheap free-lunch levers are exhausted. Next cycle
should test estimate-sharpening edits on the wide-sketch base rather than
turning any saturated size knob further.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

A decisive lever-exhaustion cycle: all three variants branched off
cycle-0005/v001 (compact layout + wide sketch) and each attacked one of
the three surfaces the cross-cycle log had named as the only live
hit-rate movers — the estimate rule (conservative update, v001), the
admission decision (a one-count margin, v002), and sketch geometry
(width 16→32, v003). None produced a real hit-rate gain; v002's margin
nudged it up by a hair, the other two landed flat. They are Winners only
in the cost-axis sense, not on hit-rate.

## Patterns

- The three named hit-rate levers are now all spent on the wide base.
  Conservative update's factor-4 gain does not transfer to factor-16
  (the wide table already suppressed the aliasing it corrects), width is
  saturated (doubling buys only cache-line tax), and a single-count
  admission margin rarely flips a decision on a hot set whose
  frequencies are separated by far more than one count. Each probe
  closed its surface negatively.
- The binding constraint is now isolated by elimination, not by a win:
  it is neither the admission margin nor the increment rule nor more
  columns. The hot set's true frequencies are already cleanly separated
  at this geometry, so sharpening, hysteresis, and width all push on a
  constraint that is no longer binding.
- Cost asymmetry is now the rule for any estimate/geometry edit: every
  probe that touched the hot path (conservative min-scan, wider stride)
  paid unconditional per-op time for zero hit-rate, while the admission
  margin — which only runs on the full-cache eviction branch — was
  genuinely free but also nearly inert.

## Open questions

- With all three logged hit-rate surfaces ruled flat on the wide base,
  is there *any* remaining edit that moves hit-rate, or has this trace's
  hit-rate effectively saturated at the incumbent base?
- If hit-rate is saturated, does the frontier shift entirely to banking
  cost-axis savings (footprint, tail, per-op) on a fixed hit-rate?

## Continuation

Stop turning the three named hit-rate levers: conservative update is now
ruled flat on the wide base (do not re-test it here), sketch width is
saturated at factor 16, and a small admission margin is inert on this
stable hot set. Cycle 0007 already retired aging, depth, and
sample-depth as accuracy levers. The cross-cycle log no longer points to
a credible hit-rate mover — next cycle should either hunt a genuinely
new admission/eviction *structure* (not a margin or constant tweak) or
accept hit-rate as saturated and pivot to banking the confirmed
cost-axis wins (footprint, p99 tail) at fixed hit-rate.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Acting on Cycle 0008's "hunt a new structure or bank cost wins" mandate,
the three variants split across surfaces: a deeper cost cut (sketch depth
4→2, v001), a genuinely new eviction structure (SLRU probation/protected
split, v002), and a microarchitectural increment swap (branchless
saturating add, v003). Every variant missed its primary claim — the two
cost probes regressed per-op-time and v001 also lost hit-rate; the SLRU
moved hit-rate only a sliver while charging footprint and per-hit work.
They are Winners only in the narrowest cost-axis sense.

## Patterns

- The increment loop body is confirmed a dead cost surface. Both edits
  that touched the per-row bump — removing rows (v001) and removing the
  branch (v003) — made the hot path *slower*, not faster. The saved work
  sits below noise, and on this saturating, well-predicted counter
  distribution the branchless rewrite forces an unconditional store where
  the branch let the common path skip it. Per-op gains must come from the
  eviction walk, not the sketch increment.
- Sketch depth is not actually accuracy-saturated at the bottom end.
  Cutting 4→2 leaked aliased estimates into admission and dropped
  hit-rate: the parent's extra rows were still doing real
  collision-correction. Depth=4 is the operating floor, not a saturated
  knob to trim.
- A real new eviction structure still hits the same ceiling. SLRU
  shielded twice-proven entries from one-hit newcomers exactly as
  intended, yet the existing `incoming_freq > victim_freq` admission gate
  already filters those newcomers — so list structure contributed almost
  nothing. The hit-rate ceiling is set by admission, not by list shape.

## Open questions

- With admission confirmed (again) as the binding constraint, is there
  any admission-side edit that isn't a margin/constant tweak — already
  ruled inert — that could move hit-rate?
- Are there *any* cost-axis savings left to bank that don't tax the hot
  path, now that depth, the increment loop, and list structure all carry
  a per-op or hit-rate cost?

## Continuation

Close three more threads: sketch depth has a hard floor at 4 (do not
trim it), the increment loop body is a confirmed non-surface for per-op
speed (neither fewer rows nor branchless bump helps), and SLRU/segmented
eviction is ruled flat because admission already does its job. Cycle 0008
already retired width, conservative update, and the admission margin.
The hit-rate-mover search is now exhausted across estimate, geometry,
admission-margin, and eviction-structure surfaces — treat hit-rate as
saturated at the incumbent base. Next cycle should either probe the one
untouched lever (the eviction *walk* / victim sampling, named by v003 as
the remaining cost surface) for tail/per-op savings, or stop hunting
hit-rate entirely and bank fixed-hit-rate cost wins.


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
