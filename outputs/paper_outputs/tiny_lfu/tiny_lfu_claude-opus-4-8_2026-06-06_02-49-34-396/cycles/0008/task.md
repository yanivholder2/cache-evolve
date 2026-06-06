# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0008

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
