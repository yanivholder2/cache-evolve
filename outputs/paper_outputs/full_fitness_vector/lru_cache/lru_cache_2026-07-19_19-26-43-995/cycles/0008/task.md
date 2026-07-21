# Task: cycle-insight-writer

Cycle: 0008
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0008

## Winners

(none)

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0008\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0008\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0008\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0008\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0008\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0008\hypotheses\H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
