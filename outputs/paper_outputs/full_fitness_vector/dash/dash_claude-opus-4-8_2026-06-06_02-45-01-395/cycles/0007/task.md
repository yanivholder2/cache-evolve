# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0007

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants forked from the cycle-0005 v001 side-table parent and
attacked the same documented bottleneck — the `target_heat` side-table probe
that bounds that variant's get latency — by three routes: swap SipHash for a
trivial `u64` hasher (v001), pre-reserve capacity to kill rehash spikes
(v002), and replace the hash with a position-aligned per-bucket `Vec<u8>`
(v003). All three entered the Winners, but only v001 was a clean win: it cut
all four op-latency metrics with no regression and broke below the parent's
get-p50 floor at the metadata corner — proving cycle 0005's "frontier may be
converged" call was premature.

## Patterns

- The side-table cost was the *hasher*, not the hash structure. The
  bottleneck cycle 0005 pinned on this probe was SipHash overkill on
  integer keys; removing the keyed permutation while keeping the `HashMap`,
  the gate decisions, and the layout byte-for-byte moved get-p50 below the
  parent floor for free. This is the locality-preserving latency win the
  prior cycle's open question asked for — and it lived inside the existing
  structure, not in a new one.
- Tail and median latency on this probe have separable causes. The hasher
  swap (v001) moved p50 and both tails together; capacity pre-reservation
  (v002) cut only the get p99 rehash spike and left put p99 untouched —
  confirming put's tail is gated by a different mechanism the side table
  can't reach. Median is hasher-bound; the get tail is rehash-bound; the put
  tail is neither.
- Eliminating the hash entirely backfired (v003): a position-aligned
  parallel `Vec<u8>` must ride along with its item through every ClassicLRU
  remove+push, paying an O(n) element shift on *every* hit versus the
  parent's target-only hashed probe — slower, not faster, and the alignment
  also failed to reproduce the promotion-gate semantics exactly, nicking
  hit-rate. The hash was never the bottleneck; the *keyed permutation* on top
  of it was.

## Open questions

- v001 (hasher swap) and v002 (capacity reservation) are orthogonal by
  construction — one hits median, the other the get tail. Do they compose
  into a single variant that wins both, unlike the substitutive stacks of
  cycle 0005?
- The put p99 tail resisted every lever this cycle. What mechanism on the put
  path bounds it, and is it reachable at all?

## Continuation

The frontier was not converged: trivializing the `u64` hasher on the
side-table probe is the new latency baseline at the metadata corner — carry
it forward. Because v001 and v002 target disjoint causes (median hasher cost
vs. get-tail rehash cost), next cycle should try composing them; this is a
genuinely additive pair, not the substitutive stack that failed in 0005.
Drop the parallel-array route (v003) — replacing a hash with an O(n)
under-reorder shift is a locality regression. Treat hit-rate as still fully
capped, and probe the put-p99 tail as the one unexplained latency source.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
