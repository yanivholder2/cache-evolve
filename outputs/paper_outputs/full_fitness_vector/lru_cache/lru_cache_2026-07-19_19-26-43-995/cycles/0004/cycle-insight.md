# Cycle 0004 Insight

## Cohort outcome

All three variants pursued policy-layer improvements — adaptive quota sizing, CLOCK-style lazy relinking, and a two-hit probationary filter — yet none produced a Winner; every mechanism paid unconditional per-op overhead for logic that barely fired or provided negligible quality benefit on this workload.

## Patterns

- **Workload stability defeats adaptive mechanisms.** The Zipfian trace's hot set is so stable that adaptive quota sizing never observed enough demotion pressure to trigger a single adjustment epoch, yet paid the counter-increment and branch on every operation. Mechanisms designed to respond to workload phase shifts are wasted on a stationary distribution.
- **CLOCK-style deferred relinking was rejected for the second time.** In Cycle 0001 a CLOCK variant lost due to hand-sweep cache-line pollution; here a bitmap-backed CLOCK variant lost due to a 24-byte `Vec` header bloat and the same sweep cost at demotion time. The underlying trade-off is unfavorable: amortising relinking only pays when promotions vastly outnumber demotions, but on a full cache the two rates are coupled.
- **The two-hit filter is directionally correct but miscalibrated for this trace.** Encoding the second-chance flag in an existing spare field costs no layout bytes, which is the right structural approach. The marginal hit-rate improvement confirms one-hit graduates do exist. However, the splice on every first probationary hit means the optimization path is hit on the common case (first access) rather than the rare case (second access), inverting the cost/benefit ratio.
- **All three variants regressed per-op-time by the same tier.** The consistent p50 movement from near-zero to the ~100 ns tier across structurally unrelated designs suggests the workload's tight per-op budget leaves no room for extra branching or memory indirection regardless of how rarely the branch fires.

## Open questions

- Could two-hit graduation be restructured so the extra work fires only at the second access (the graduation decision), not the first? For example, marking the SEEN_BIT without relinking on the first hit — paying only a single bit-or rather than a list splice.
- Is there a non-policy structural change — such as arena alignment or slab grouping of nodes — that reduces cache-miss cost on `get` traversal without adding any per-op branch?
- The key-clone elision hypothesis remains untested for non-`Copy` key types. Does validating it on a `String`-keyed workload finally produce a clean put-path win?

## Continuation

The policy-layer search space around the current 2Q baseline appears largely exhausted: adaptive quota, CLOCK second-chance, MRU-tail guard, and two-hit admission filtering have all failed to improve the per-op-time/hit-rate trade-off on this workload. The next cycle should shift to two orthogonal axes: (1) a structural memory-layout experiment that reduces node-access cache-miss cost without any per-op branch — for example, packing nodes into a denser arena or aligning hot fields to cache-line boundaries; and (2) a `String`-keyed benchmark fixture to finally test key-clone elision under conditions where clone cost is non-trivial. Policy experimentation should pause until a structural win re-opens headroom.
