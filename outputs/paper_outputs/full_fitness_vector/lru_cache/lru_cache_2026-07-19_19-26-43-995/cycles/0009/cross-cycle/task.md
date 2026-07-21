# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants targeted node memory compaction or hit-rate improvement via structural redesign; only v002's sentinel-based link compression landed as a clean winner, delivering smaller nodes and better get latency without sacrificing eviction quality.

## Patterns

- **Metadata compaction was the most reliable lever.** Both v002 (u32 sentinel links) and v003 (CLOCK flat slots) halved node size — but compaction that preserved the LRU chain (v002) also improved get latency through better cache-line utilization, while compaction that eliminated the chain (v003) introduced a new bottleneck via the CLOCK hand's sweep-induced cache-line pollution.
- **Algorithm complexity adds hidden per-op cost.** The two-queue variant (v001) and CLOCK (v003) both introduced extra pointer work or array traversal that eroded or negated their structural gains. The simpler structural change (v002) outperformed both.
- **Exact LRU ordering remains valuable for this workload.** v001's two-queue policy improved hit rate by protecting hot items, but v003's coarser single-bit recency signal hurt hit rate — suggesting the workload has enough reuse that precision in eviction decisions pays off.
- **Put latency is a secondary but real concern.** v002's slight put regression from cast overhead and v003's put blowup from CLOCK sweeps both appeared; get-path improvements should not be assumed to carry over to put.

## Open questions

- Can the two-queue hit-rate gain from v001 be recovered on top of v002's leaner node layout, where the extra per-queue bookkeeping fields represent a smaller fractional overhead?
- Is CLOCK's cache-line pollution inherent to the algorithm, or does a segmented or lazy-sweep variant recover the structural win without the latency penalty?
- How much further can node size be reduced (e.g., toward a single cache line per node) before diminishing returns set in?

## Continuation

v002's u32 sentinel-based links are the new structural baseline — next cycle should treat this as the parent and compound on it. The most promising directions are: (1) revisiting the two-queue policy on top of v002's leaner nodes to see whether hit-rate gains now come at an acceptable latency cost, and (2) exploring further struct packing to push node density higher. CLOCK-style approaches should be deprioritized unless a mechanism to avoid hand-sweep cache pollution is proposed.


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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle tested three compositions built on the link-array structural baseline: a standalone quota enlargement (v001), quota enlargement paired with SEEN_BIT two-hit admission (v002), and SEEN_BIT with an eviction-path second-chance rescue (v003). Only the pure quota change won; both SEEN_BIT variants hurt hit-rate.

## Patterns

- **Quota enlargement is validated and cost-free.** v001 cleanly reproduced the hit-rate improvement first seen in Cycle 0005/v003 and confirmed that the apparent latency cost in that earlier run was measurement noise from an anomalous zero-floor baseline. The 87.5% protected share is now a settled, noise-free result.
- **SEEN_BIT's two-hit gate starves the policy it is meant to serve.** In v002, enlarging the protected pool offered no benefit because the stricter admission barrier prevented enough items from graduating to fill it. The mechanism's precondition — sufficient repeat accesses within the probationary window — is not met by this workload's access pattern.
- **Eviction-path rescue is undermined by SEEN_BIT reset.** v003's `push_back_prob` call unconditionally cleared SEEN_BIT, raising the effective graduation barrier from one additional hit to two for every rescued item. The rescue compounded the gate's over-strictness rather than softening it, making the displaced items net-more-valuable than the saved ones.
- **All two-hit mechanisms have now failed across three cycles.** Cycle 0004 (SEEN_BIT splice), Cycle 0005 (cost-free bit-OR), and Cycle 0006 (composition and rescue) each exposed the same root cause: workload-a does not generate enough within-window repeat accesses for a two-hit requirement to be net beneficial.

## Open questions

- Is there a softer, position-based or frequency-sketch-based admission variant that loosens the gate without fully abandoning selective graduation?
- With the quota now at 87.5% protected, is there additional headroom by pushing it further, or does the probationary pool's minimum useful size create a floor?
- Could directly measuring the probationary graduation rate (fraction of items reaching a second hit before eviction) guide the choice of admission threshold, rather than inferring it from aggregate hit-rate?

## Continuation

The next cycle should adopt v001's quota policy as the new structural baseline and retire SEEN_BIT entirely — three cycles of failure on this workload is conclusive. Two directions worth probing: (1) experiment with quota values beyond 87.5% to locate the point of diminishing returns or reversal; (2) explore a lighter admission filter, such as a compact frequency sketch that graduates any item with an estimated frequency above a low threshold, rather than requiring two sequential hits within a narrow window. Do not revisit the eviction-path rescue or any mechanism that requires a per-item state bit tracking the number of probationary hits seen.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants continued the quota-compression line or probed lightweight admission filters; only the pure quota reduction (v001, 93.75% protected) won, extending the now four-step monotonic hit-rate improvement. Both the CountMin sketch and the demotion-reordering change failed to add anything on top of the current policy baseline.

## Patterns

- **The quota-compression trend is monotonic but visibly flattening.** Each successive halving of the probationary pool (75% → 87.5% → 93.75% protected) has improved hit-rate, but the magnitude of each step is shrinking, signalling approach toward a plateau. The next halving (capacity/32, ~3% probationary) may yield only marginal gain before reversal.
- **A persistent CountMin sketch provides no discrimination on this workload.** v002 tested the hypothesis that cross-visit frequency history would outperform the within-window SEEN_BIT by fast-tracking returning items. It did not — the Zipfian moderate-frequency population already graduates effectively under immediate one-hit promotion; adding a freq≥2 gate and 64 bytes of struct overhead changed nothing. The result closes the "lighter filter" branch opened in Cycle 0006: workload-a's access pattern does not create enough graduation-quality discrimination at any reasonable threshold.
- **Demotion ordering is inert at high protected quotas.** v003 confirmed that placing overflow-demoted items at the LRU end of probationary rather than the MRU end has no measurable effect when the protected queue overflows only rarely. The mechanism requires frequent overflow to matter; at 93.75% protected the overflow event barely fires.
- **Per-op-time measurement noise from the anomalous zero-floor parent continues to inflate apparent regression.** Both v001 and v003 showed a p50 jump that the variants' own insights attribute to baseline anomaly resolution rather than introduced cost. This pattern has recurred in multiple cycles; new winning variants should be measured relative to a clean non-zero-floor parent before latency verdicts are drawn.

## Open questions

- Is there a minimum viable probationary pool size below which the cache can no longer replenish the protected queue with enough diversity? At capacity/32 the probationary budget is ~3%; does the distribution of cold-miss items require more runway than that?
- The frequency-sketch concept failed when used as a graduation gate. Could a sketch used instead for eviction ordering within the probationary queue — evicting estimated low-frequency items first rather than strict FIFO — provide benefit without the graduation-gating overhead?
- Is the flattening hit-rate curve evidence that the quota axis is nearly exhausted, or does it reflect a workload-specific saturation point that would shift on a different access distribution?

## Continuation

The next cycle should treat v001 (93.75% protected, capacity/16 prob_cap) as the new baseline and continue the quota exploration: test capacity/32 (~3% probationary) to locate the reversal point or confirm diminishing returns are steepening toward zero. In parallel, probe one structurally different direction to avoid over-indexing on a single axis: the node link-array layout from Cycle 0005/v002 was cost-neutral but never combined with the current deep-quota baseline — testing that composition may reveal latency benefits that were below the measurement floor at lower protected shares. Do not revisit admission filters (SEEN_BIT, CountMin) or demotion ordering; both branches are now conclusively closed on this workload.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle probed the final steps of quota compression (capacity/32), a frequency-guided 2-candidate eviction scheme, and a cross-parent validation of the 93.75% quota on the link-array structural baseline. No variants won; the most significant result is a verdict reversal: the per-op-time cost of the high protected quota is real, not a measurement artifact.

## Patterns

- **The per-op-time regression is quota-induced, not measurement noise.** v003 applied the same 93.75% quota change to a confirmed clean-floor parent (the link-array baseline) and reproduced both the hit-rate gain and the latency penalty. This retroactively recharacterizes prior cycles' "winners" at capacity/16: they carry a genuine per-op-time trade-off that was obscured by anomalous zero-floor baselines, not escaped by parent selection.
- **Quota compression is in steep diminishing returns on hit-rate.** v001's step from capacity/16 to capacity/32 delivered a visibly smaller improvement than each prior halving, confirming the trend flagged in Cycle 0007 is steepening toward a plateau or reversal.
- **Frequency-guided eviction requires probationary runway it cannot have at high quotas.** v002 closed the last open branch from Cycle 0007: at capacity/16 probationary size, items are evicted before the flat frequency table accumulates distinguishable signal, making the 2-candidate comparison a coin flip. Frequency hints and deep-quota operation are structurally incompatible under this workload's access pattern.
- **The 93.75% quota now has a confirmed cost on every tested parent.** The combination of high hit-rate and zero latency penalty has not been achieved at this quota level; both signals cannot be won simultaneously with the current policy structure.

## Open questions

- Is there a quota level — somewhere between the current baseline (87.5%) and the best hit-rate setting (93.75%) — where the LRU boundary enforcement overhead remains below the measurement floor while still capturing a fraction of the hit-rate gain?
- The latency cost is linked to increased overflow frequency (demotion from protected to probationary). Could batching or amortizing demotion events reduce the per-op penalty without changing the effective quota?
- With quota compression, admission filtering, and frequency-guided eviction all exhausted, what structural or algorithmic axes remain unexplored on this workload?

## Continuation

The next cycle should treat the 87.5% quota (capacity/8 probationary) as the confirmed clean baseline — it is the last setting known to be cost-free on both dimensions — and accept that moving to 93.75% carries a real latency penalty. Two directions worth testing: (1) probe an intermediate quota (e.g., capacity/12 or capacity/24) to determine whether a smaller overflow rate stays below the latency measurement floor while recovering part of the hit-rate gain; (2) pivot away from quota and explore a structurally different axis such as reducing hash-map lookup overhead or node memory layout changes that could improve per-op-time independently of policy. Do not revisit admission filters, frequency sketches, or demotion ordering — all are conclusively closed. Do not push quota beyond capacity/16 without first resolving the latency trade-off at that level.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants attempted to recover hit-rate gains beyond the 87.5% protected quota without incurring a latency penalty: one probed an intermediate quota (capacity/12, ~91.67%), one batched demotion events at the known-costly 93.75% quota, and one combined a faster FNV-1a hasher with the 93.75% quota to try to offset demotion overhead. None won — every variant improved hit-rate but each still regressed per-op-time above the measurement floor.

## Patterns

- **The zero-latency ceiling for quota compression is below 91.67% protected.** Even the capacity/12 intermediate step — the smallest quota increase tested — pushed get p50 above the floor. The safe operating range for per-op-time does not extend past 87.5%, and the gap between the last cost-free point and the first regressing point is narrow enough that finer-grained probing is unlikely to find a free lunch.
- **Demotion batching is defeated by quota-induced graduation frequency.** v002's batch-2 scheme was designed to halve demotion event frequency, but the tighter probationary window simultaneously raised graduation rates — more items earn promotion per unit time — so the net demotion burden on the hot path was unchanged or worse. Amortisation cannot offset a mechanism that increases the event rate it is trying to spread.
- **FNV-1a delivers a genuine structural benefit but insufficient latency headroom.** v003 confirmed a real 16-byte struct-size reduction from replacing the seeded SipHash builder with a zero-sized FNV type, and hash computation cost did fall. However, the demotion churn at 93.75% quota is the dominant cost driver and is larger than the hashing savings — the hasher speedup is real but not wide enough to absorb the policy overhead.
- **The hit-rate vs. per-op-time trade-off at quota > 87.5% is now confirmed from three independent angles.** Intermediate quota (v001), event batching (v002), and hash speedup (v003) all failed to break the coupling. The trade-off appears structural to this policy design rather than fixable by tuning one mechanism.

## Open questions

- FNV-1a has only ever been tested paired with quota increases; its net effect at the confirmed clean 87.5% baseline is unknown. Could it lower per-op-time on the current best parent, potentially creating headroom to absorb a modest future quota increase?
- With the quota axis exhausted and latency-reduction mechanisms (batching, faster hashing) insufficient at higher quotas, are there node memory-layout or cache-line alignment changes that could reduce the demotion cost itself rather than trying to offset it?
- Is there a fundamentally different policy structure — such as segmented LRU with lazy promotion rather than eager demotion — that achieves a similar effective protected ratio without the per-operation enforcement overhead?

## Continuation

The next cycle should accept 87.5% (capacity/8 probationary) as the permanent quota ceiling under the current policy structure and stop probing higher values — three independent strategies have failed to break the latency coupling. Two directions worth pursuing: (1) apply FNV-1a to the clean 87.5% baseline alone, as a pure per-op-time optimization never tested without a quota change; if it lowers latency at the current best point, it creates a new floor that future quota experiments can target. (2) Pivot to a structural axis — node memory layout, cache-line packing, or a policy redesign that avoids eager per-operation demotion enforcement — rather than continuing to tune the existing protected/probationary split. Do not revisit demotion batching, admission filters, or quota values above 87.5%.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
