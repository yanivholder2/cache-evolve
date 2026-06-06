# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0005

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cohort cashed in cycle-0001's two safe directions and answered its
open question. All three variants entered the Winners: two pushed `Item`
footprint to its floor (v001 stacks v003's slim `Item` onto v001's MRU
promotion; v002 deletes the last metadata field outright), and one
(v003) widened the promotion gate from "single hottest" to a proportional
top-quarter window, lifting hit-rate above the v001 incumbent without
re-introducing v002-cycle-0001's churn.

## Patterns

- Placement and footprint proved genuinely orthogonal: porting the slim
  `Item` onto the MRU-promotion winner added both wins (smaller entries,
  faster ops) while leaving hit-rate untouched, confirming cycle-0001's
  bet that the two axes don't trade against each other.
- Metadata slimming bottomed out this cycle. Once `timestamp` was gone,
  the lone frequency counter was the only thing padding `Item<u64,u64>`
  past its bare key+value core; deleting it collapsed the alignment
  padding for free because the active positional `ClassicLRU` never read
  the field. There is little metadata left to remove.
- The middle-ground promotion gate worked, but the gain was small — the
  bulk of protectable warm mass on this trace already sits near the MRU
  head, so a quarter-window captures most of what the tight gate missed.
  Widening also shortened target buckets, so latency improved alongside
  hit-rate rather than trading against it.

## Open questions

- Does combining the slim `Item` (H01/H02 footprint floor) with the
  top-quarter promotion gate (v003) compound cleanly, or does the
  shorter-bucket effect of the wider gate change the placement calculus?
- The quarter-window gain was small — is there a better-shaped gate
  (different fraction, or recency-weighted rather than positional) that
  reaches the warm keys the window still misses, or is the trace's
  protectable mass simply near-exhausted?
- With `Item` now at its byte floor, where does the next size win come
  from — bucket/segment structure rather than per-entry layout?

## Continuation

Carry forward the merged best: slim `Item` at its 16-byte floor plus the
top-quarter promotion gate, as the new baseline to beat. Treat further
per-entry metadata trimming as exhausted — pivot the size axis to
container/segment structure instead. On hit-rate, the positional window
is near diminishing returns; next cycle should test gate *shape* (varying
the fraction or trying a non-positional weighting) rather than just gate
*width*, and probe whether placement and the wider gate still stack when
applied together.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants forked from the cycle-0002 slim+MRU base and probed the
promotion-gate / hot-path axis: a wider positional window (v001), a
frequency-driven gate spending the free `lfu_counter` (v002), and a pure
branch-reorder latency play (v003). All three entered the Winners, but this
was a cohort of near-null results — every change landed within or below the
trace/measurement noise floor, with no clean compounding win among them.

## Patterns

- The promotion-gate axis has saturated on this trace. Widening the gate
  (v001) and switching it from positional to frequency (v002) each nudged
  hit-rate up only marginally, because MRU placement (`InsertPosition::Back`)
  already captures the warm mass either lever would protect — gate shape and
  MRU placement are substitutes, not additive. This directly answers cycle
  0002's open questions 1 and 2: protectable warmth is near-exhausted.
- Touching the hot `get` path to chase hit-rate carries real latency cost
  with little payoff: both the wider gate's promotion cascade (v001) and the
  per-hit counter read-modify-write (v002) raised p50 for a hit-rate gain
  indistinguishable from noise.
- Micro-latency reorders are below this harness's resolution. v003's branch
  reorder strictly removes a scan per hit yet moved latency the wrong way —
  and even `put`, which the change can't touch, regressed — confirming the
  saving is swamped by variance.

## Open questions

- With both placement and gate shape exhausted on hit-rate, is there any
  remaining lever on this trace that isn't noise-limited, or is hit-rate
  fully tapped?
- Since the harness can't resolve single-scan-per-hit savings, what
  granularity of structural change *is* large enough to measure?

## Continuation

Stop tuning the promotion gate and hot-path probe order — that axis is
saturated and its effects sit below the noise floor; further gate-shape or
reorder variants are not worth a slot. Pivot decisively to the
container/segment structure for the next real win (the size axis flagged in
cycle 0002), and size any future latency hypothesis to clear the measurement
noise floor rather than chasing per-hit micro-savings.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Acting on cycle-0003's mandate, this cohort abandoned gate tuning and
attacked three structurally distinct levers: a segment-owned side table to
break the size/hit-rate trade (v001), hot-path devirtualization of the
eviction policy (v002), and raised set-associativity for hit-rate (v003).
All three entered the Winners, and together they resolved the cycle's open
questions — one reaching the long-sought Pareto corner, one establishing what
the harness can actually measure, and one cleanly falsifying a miss-class
hypothesis.

## Patterns

- The container/segment pivot paid off where per-entry tuning could not.
  Relocating the promotion-heat signal out of `Item` into a segment-side map
  (v001) finally occupied the open Pareto corner — the incumbent's best
  hit-rate held bit-for-bit AT the metadata floor that the counter-carrying
  parent could not reach. The structure-vs-layout split flagged since cycle
  0002 was the right frame.
- Structural latency change *is* resolvable; micro-reorders were not.
  Devirtualizing the trait-object call and enabling remove+push inlining
  (v002) moved median/get-tail latency above the noise floor that swallowed
  cycle-0003's branch reorder, directly answering "what granularity is
  detectable."
- Hit-rate is now firmly capacity/compulsory-bound, not conflict-bound.
  Widening the neighborhood to 3-wide (v003) left hit-rate bit-for-bit
  unchanged, falsifying the conflict-miss hypothesis and closing
  associativity as a dead lever — alongside the already-saturated gate and
  placement axes.
- Side tables trade tail latency for size: v001's free struct read became a
  hash-and-probe, pushing p99 up. Relocation is not a free substitute.

## Open questions

- With placement, gate shape, and associativity all exhausted on hit-rate,
  is any remaining lever capable of moving it, or is it fully capped by the
  trace's compulsory/capacity mass?
- Can the v002 devirtualization win and the v001 side-table relocation be
  combined, or does the side-table's added hot-path hashing eat the
  devirtualization latency gain?
- Are there further structural latency wins of v002's magnitude (allocation
  strategy, bucket layout), and can any reduce p99 rather than just p50?

## Continuation

Carry forward v002's devirtualized hot path as the new latency baseline — it
is the clearest, harness-resolvable win available. Treat hit-rate as capped:
stop spending slots on gate, placement, or associativity variants. For size,
v001's side-table reaches the Pareto corner but at a tail-latency cost worth
revisiting; next cycle should test whether v002's latency win and v001's size
win stack, and keep size/latency hypotheses sized to clear the noise floor.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
