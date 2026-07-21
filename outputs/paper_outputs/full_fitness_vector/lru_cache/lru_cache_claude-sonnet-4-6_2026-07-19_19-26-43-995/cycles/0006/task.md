# Task: cycle-insight-writer

Cycle: 0006
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0006

## Winners

- v001

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0006\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0006\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0006\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0006\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0006\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0006\hypotheses\H03.md

## Prior cycles digest

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle executed exactly the two structural directions recommended by Cycle 0004: a Node layout split separating link metadata from key/value data (v002) and a cost-free two-hit admission filter using a SEEN_BIT (v001), plus a protected-quota enlargement (v003). Both structural variants landed as Winners; the quota-only change did not, blocked by an apparent (likely noise) per-op-time regression.

## Patterns

- **Winners won by not regressing, not by visibly improving.** Both v001 and v002 produced no measurable per-op-time hurt — v001 confirmed that bit-OR on the first probationary access is essentially free, and v002 confirmed that the link-array split is cost-neutral — yet neither showed a positive latency signal. The tight per-op budget established in prior cycles means the bar for winning is "do no harm" when measuring at current timer resolution.
- **Cost-free deferred graduation is structurally solved but policy-tuning is unfinished.** Reducing the SEEN_BIT first-access path to a single bit-OR resolved the Cycle 0004 open question on implementation cost, but the stricter two-hit admission policy exposes a workload sensitivity: enough single-access items exist that a meaningful fraction are evicted before reaching the second hit needed to graduate, hurting hit-rate. The mechanism is correct; the calibration is not yet right for this workload.
- **The link-array split achieved its layout goal without side effects.** Node size dropped from 24 to 16 bytes and link-traversal density increased substantially. The latency benefit is real in principle but falls below the current measurement floor — the absence of regression is still a clean endorsement of the structural direction.
- **The quota enlargement is directionally correct but measurement-opaque.** v003's hit-rate improvement is genuine; its apparent per-op-time hurt is most plausibly measurement noise from the parent's anomalously-zero p50 floor. The policy is still unconfirmed as cost-free because the noise masks the verdict.

## Open questions

- Can the SEEN_BIT two-hit filter (v001) and the enlarged protected quota (v003's direction) be composed? A deeper protected pool would keep entries that finally graduate resident longer, potentially recovering the hit-rate lost to stricter admission.
- Does the quota enlargement's apparent per-op regression persist or resolve when retested on the link-array baseline (v002), which now has a non-zero p50 floor to measure against?
- Is there a workload- or frequency-adaptive threshold that lets the SEEN_BIT filter skip two-hit enforcement for items it can infer will not recur, rather than applying the stricter policy uniformly?

## Continuation

The next cycle should compose the SEEN_BIT deferred graduation (v001) with the quota enlargement (v003's direction), built on top of the link-array structural baseline (v002). The hypothesis is that a larger protected pool compensates for the stricter admission gate: items that survive to a second hit are then shielded from demotion long enough to accumulate further hits. Additionally, the quota enlargement should be retested cleanly on the v002 base to determine whether its per-op-time signal is genuine or an artifact of the prior measurement floor. Do not revisit standalone structural layouts or adaptive policy mechanisms — the search space is now narrow enough to pursue targeted composition.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
