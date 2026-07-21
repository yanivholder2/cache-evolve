# Task: cycle-insight-writer

Cycle: 0010
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0010

## Winners

- v001

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0010\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0010\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0010\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0010\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0010\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0010\hypotheses\H03.md

## Prior cycles digest

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

Exactly one file: `cycle-insight.md` in the output dir.
