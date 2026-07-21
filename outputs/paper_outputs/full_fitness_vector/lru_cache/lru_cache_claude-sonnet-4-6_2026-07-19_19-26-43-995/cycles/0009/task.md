# Task: cycle-insight-writer

Cycle: 0009
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0009

## Winners

(none)

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0009\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0009\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0009\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0009\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0009\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0009\hypotheses\H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
