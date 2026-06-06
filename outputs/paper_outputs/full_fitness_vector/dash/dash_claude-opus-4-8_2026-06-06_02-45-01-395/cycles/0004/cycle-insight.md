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
