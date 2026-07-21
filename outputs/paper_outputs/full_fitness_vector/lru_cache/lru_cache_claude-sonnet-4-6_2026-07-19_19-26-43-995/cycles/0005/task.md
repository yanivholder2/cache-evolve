# Task: cycle-insight-writer

Cycle: 0005
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0005

## Winners

- v002
- v001

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0005\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0005\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0005\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0005\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0005\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0005\hypotheses\H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two independent directions both landed as Winners: v001 layered a two-queue eviction policy onto the u32-sentinel baseline without growing the node, and v002 short-circuited the get promote path for already-MRU nodes. v003's attempt to accelerate put via the `entry()` API and forced inlining backfired entirely.

## Patterns

- **Policy and algorithmic shortcuts beat micro-optimization.** Both winners applied meaningful behavioral changes (2Q promotion logic, MRU-tail guard); the loser applied low-level code hints (`#[inline(always)]`, `entry()` API) that either couldn't reach their target path at steady state or expanded code size past the point where instruction-cache fit was preserved.
- **The protected-bit-in-prev encoding confirmed as practical.** Folding queue membership into a spare bit of the existing `prev` field kept Node at 24 bytes while unlocking full 2Q semantics — the cycle 0001 concern that 2Q required node growth turned out to be avoidable.
- **MRU-guard gains come with p99 and cross-path cost.** The get p50 win from skipping no-op promotes is real and large, but the added conditional disturbs branch-predictor and instruction-cache state enough to raise get p99 and put p50 — a reminder that single-path optimizations in hot mixed-workload code have side effects.
- **Forced inlining is not a free win.** `#[inline(always)]` on small helpers can push a hot path out of ideal instruction-cache placement, producing the opposite of the intended latency reduction.
- **Borrow-checker constraints cap `entry()` API gains.** On the eviction path (dominant at steady state), the occupied `VacantEntry` must be released before `map.remove()` can run, erasing the promised single-probe benefit and adding an extra pointer move instead.

## Open questions

- Can v001 (2Q) and v002 (MRU guard) be composed? The MRU guard would naturally extend to protected-queue tail hits; it may amplify the hit-rate benefit of 2Q by also cutting relinking cost on the most-recently-used protected entries.
- v002's p99 regression: is the branch-predictor disruption inherent to the guard, or can branch placement or profile-guided optimization recover the p50 gain without widening the tail?
- Is there a put-path latency improvement that does not run into the borrow-checker eviction-path obstacle? Pure structural changes (e.g., avoiding the key clone on eviction) may be safer than API-level tricks.

## Continuation

The next cycle should attempt to compose v001 and v002: apply the MRU-tail guard on top of the 2Q layout, since both are structurally compatible and target different bottlenecks (hit quality vs. promote cost on the hot path). The `#[inline(always)]` / `entry()` micro-optimization path should be dropped — the borrow-checker and I-cache constraints make it a poor fit. If a third variant is explored, focus on reducing key-clone cost during eviction rather than hash-probe count, as that is the remaining identifiable per-put cost not yet addressed.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants executed exactly the directions suggested by the previous cycle's continuation — composing 2Q with the MRU-tail guard, tuning the protected-queue quota, and eliminating key-clone cost on eviction — yet none produced a clean Winner; every variant either regressed latency or improved hit-rate at the cost of higher per-op time.

## Patterns

- **The MRU-tail guard does not generalize across workloads.** Cycle 0002's guard delivered a win on its own workload, but when composed with the 2Q layout on workload-a, the guard fires too rarely: the branch and extra pointer read add cost on every protected hit while the bypass only triggers for the rare exact-tail case. Guard-style shortcuts only pay off when their fire rate is high enough to amortize the check.
- **Protected-quota tuning is a genuine but costly lever.** Expanding the protected budget from 75% to 87.5% of capacity measurably reduced demotion pressure and improved hit-rate on a Zipfian trace, confirming the hypothesis was directionally correct. The price is proportionally more expensive unlink/push_back_prot operations, meaning quality and throughput move in opposite directions as the quota shrinks.
- **Clone-elision must be validated against the actual key type.** The key-clone optimization had no effect because the benchmark uses a `Copy` key where `.clone()` is a register copy. The idea remains sound for heap-owning key types but cannot be confirmed or denied by the current trace setup.
- **No composition of the cycle-0002 winners produced a net win.** The two prior winners targeted orthogonal bottlenecks, but layering them introduced cumulative branch and path-length overhead that outweighed the individual gains on this workload.

## Open questions

- Is there a workload or access pattern where the protected-quota expansion's hit-rate gain dominates its latency cost, making v002 a net winner by a different metric weighting?
- Can the guard fire rate for the MRU-tail shortcut be increased by extending it to the probationary tail as well, or is workload-a simply too diffuse for any single-entry bypass to matter?
- What is the correct benchmark fixture to evaluate key-clone elision — a `String` or `Arc<str>` key trace — and does the eviction-path refactor actually reduce allocations there?

## Continuation

The next cycle should step back from composition and micro-tuning and instead probe structural changes that are independent of the current 2Q + guard stack. Priority directions: (1) evaluate whether adaptive quota sizing (dynamically shifting the protected/probationary boundary based on observed hit rates) can capture the hit-quality benefit of v002 without permanently enlarging the more-expensive protected path; (2) benchmark the key-clone elision against a non-`Copy` key type before deciding whether to carry it forward or discard it. Do not revisit the MRU-tail guard in isolation — its fire rate is workload-dependent and the current trace does not justify the check cost.


### Cycle 0004

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
