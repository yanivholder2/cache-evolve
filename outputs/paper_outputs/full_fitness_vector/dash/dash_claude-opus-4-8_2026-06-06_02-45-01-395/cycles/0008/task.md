# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0008

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants forked from the cycle-0006 v001 trivial-hasher side-table
parent and tried to wring more latency from the `target_heat` probe: fuse the
hasher and capacity wins into one (v001), swap the HashMap for a direct-mapped
array (v002), and move the per-put heat-clear to the eviction site (v003). All
three entered the Winners, but the cohort overturned the cycle's framing —
every win landed on a different op than its hypothesis named, and the get
median proved untouchable.

## Patterns

- The composition was free but mis-aimed. Adding capacity pre-reservation on
  top of the trivial hasher (v001) regressed nothing and stayed bit-for-bit on
  hit-rate and metadata, but its tail win surfaced on the **put** path, not the
  get-p99 the disjoint-cause model predicted. The get path was already pinned
  at its hasher floor with no rehash spike left to remove, so the reservation's
  only headroom was the table-growth cost that put-side hits were absorbing.
- get-p50 is now a hard floor. Both v001 and v002 left the get median flat;
  v002's structurally cheaper direct-mapped array — meant to undercut the
  SwissTable group scan — did not move it at all. Cycle 0006's hasher swap had
  already removed the only median-path cost; what remains on the side-table
  probe is tail variance, not median latency.
- The structurally cheaper table still pays off as a variance reducer. v002's
  flat array dropped both p99 tails and put-p50 while holding hit-rate exactly,
  confirming 2×-capacity sizing makes slot collisions a negligible
  promotion-timing perturbation — a safe substitute even though it doesn't
  touch the median.
- The put-p99 tail is owned by eviction/cascade work, not the side-table
  touch. v003 confirmed this negatively: the per-put `remove` was the cheapest
  possible operation (a non-resident negative lookup that short-circuits), and
  relocating it to the eviction path turned it into a real resident remove
  behind new branches, regressing both put metrics.

## Open questions

- With get-p50 pinned and the side-table probe's median cost gone, is any
  get-path median lever left, or is get latency fully converged?
- v003 located the put-p99 tail in the eviction/cascade machinery. Is that
  cascade reachable without disturbing gate decisions or hit-rate, or is it the
  irreducible cost of the demotion structure?

## Continuation

Stop targeting get-p50 — two independent routes this cycle confirmed it is at
a hard hasher floor with no probe-side headroom left. Carry forward v001
(capacity reservation, a free put-tail win) and/or v002 (direct-mapped array,
tighter p99s) as the latency baseline; they are both safe, hit-rate-neutral.
The live frontier is now the **put path**, and v003 pinpointed its p99 bound as
the eviction/cascade work, not the side table — next cycle should probe that
cascade directly rather than economizing on the per-put probe, which is already
the cheapest form of its operation. Treat hit-rate as still fully capped.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
