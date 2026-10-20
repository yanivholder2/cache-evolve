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
