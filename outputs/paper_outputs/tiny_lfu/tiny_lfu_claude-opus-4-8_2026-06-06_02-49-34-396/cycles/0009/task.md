# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0009

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
