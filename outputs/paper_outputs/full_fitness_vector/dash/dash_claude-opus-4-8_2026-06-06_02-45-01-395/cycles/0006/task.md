# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0006

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cohort tried to bank the cycle-0004 wins by composition and to close
the last open hit-rate class: stacking v002's devirtualized hot path onto
v001's side-table parent (v001), monomorphizing the outer Bucket trait
object one layer further (v002), and adding a frequency-aware ghost-list
admission lever (v003). All three entered the Winners, but the cohort was a
clean sweep of confirmatory negatives — every promised stack either failed
to compose or regressed latency, and the new lever moved nothing.

## Patterns

- The two cycle-0004 Pareto wins are substitutive on a shared hot path, not
  additive. Porting devirtualization onto the side-table parent left get-p50
  pinned at the parent's floor, because the `target_heat` hash probe — not
  the vtable dispatch — is what bounds that variant's latency. You cannot
  collect both speedups while the side-table hash sits on the same get path.
- Structural devirtualization only pays when it removes indirection without
  fattening layout. Collapsing the *outer* Bucket trait object replaced a
  compact heap pointer per slot with a fat inline struct, widening the `Vec`
  stride and regressing every per-op metric — the inverse of the inner-policy
  win. The heap pointer was buying locality; removing it cost more than the
  saved dispatch.
- Hit-rate is now confirmed capacity/compulsory-capped against every class.
  The admission/frequency lever — the one untried family — left hit-rate
  bit-for-bit at the recency-only ceiling, buying only put latency from its
  ghost bookkeeping. Placement, gate, associativity, and now admission are
  all exhausted.

## Open questions

- With composition shown to be substitutive on the shared get path, is there
  a hot-path layout where the side-table relocation and the devirtualization
  stop competing — or is the metadata-floor corner and the latency floor
  simply mutually exclusive?
- Every remaining latency lever this cycle either fattened layout or added
  bookkeeping. Is there a structural change that removes indirection while
  *preserving or improving* locality, or has latency also saturated?

## Continuation

Treat hit-rate as fully closed — admission was the last class and it moved
nothing; spend no further slots probing it. Stop attempting to stack the
cycle-0004 size and latency winners on a single hot path: they are
substitutive, and outer-layer monomorphization regresses locality. The two
banked corners (v002 devirtualized latency baseline, v001 side-table size
corner) stand as the frontier; next cycle should look for a latency change
that improves locality rather than adds indirection or inline bloat, and
otherwise accept the frontier may be converged.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
