# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0009

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-06_02-45-01-395/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants forked from the cycle-0007 incumbents and attempted
"behavior-preserving" structural moves on the dash segment: reorder the get
lookup branches stash-last (v001), relocate the per-put heat clear to the
demotion site (v002), and collapse the two-tier demotion cascade to a single
stash hop (v003). All three entered the Winners, but none was a clean win —
each move's premise was contradicted by where the work actually lives, and the
cohort overturned two standing assumptions at once.

## Patterns

- The get median lives in the **stash**, not the promoted tiers. v001 assumed
  the hottest hits resolve in target/probing and pay a wasted stash scan, so it
  moved stash last — and get-p50 regressed, because the median get hit actually
  resolves in the stash and now pays two failed scans before reaching it. The
  branch-ordering lever is real but points stash-*first*, the inverse of the
  hypothesis. Behavior was preserved exactly (hit-rate/metadata bit-for-bit);
  only the cost model was inverted.
- Relocating a probe off the median path trades median for tails. v002 did
  shave put-p50 by deleting the unconditional per-put heat clear, confirming
  that touch was real hot-path work — but the clear didn't vanish, it landed on
  the demotion branches and resurfaced as wider put-p99/get-p99 and a worse
  get-p50 summary. Moving work off the median is not free when the destination
  adds branching the tail then exposes — the same lesson cycle 0007's v003 taught
  on the eviction path.
- Hit-rate is **not** perfectly conflict-insensitive. v003 cut the put-p99
  cascade exactly as cycle 0007 predicted by collapsing the foreign-resident
  retry to a single stash hop — but hit-rate slipped below the parent, because
  the one-more-chance re-home was salvaging a small number of hits. This is the
  first crack in the multi-cycle "hit-rate fully capped / conflict-insensitive"
  framing: the demotion cascade is a genuine latency-vs-hit-rate trade, not pure
  overhead.

## Open questions

- If the get median resolves in the stash, is a stash-first reordering an
  actual get-p50 win, or does promoting the stash branch only relocate the cost
  onto the promoted-tier hits the way v002 relocated the put clear?
- The put-p99 cascade is now a priced trade: how large is the hit-rate cost of
  the single-hop demotion, and is there a cheaper conflict-mitigation step that
  keeps most of the rescued hits without the double-hop tail?

## Continuation

Two assumptions this cohort relied on were wrong and should be dropped: hot
get hits do **not** live in the promoted tiers (the median is in the stash, so
try stash-*first* if probing get-p50 at all), and the demotion cascade is **not**
free on hit-rate (collapsing it trades a few hits for the put-p99 win). The
put-p99/hit-rate trade (v003) and the put-p50/tail trade (v002) are both now
characterized as trades, not clean wins — next cycle should decide whether
either trade is worth banking rather than seeking a strictly-free version, and
should treat "hit-rate fully capped" as no longer exact in the conflict/demotion
machinery.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
