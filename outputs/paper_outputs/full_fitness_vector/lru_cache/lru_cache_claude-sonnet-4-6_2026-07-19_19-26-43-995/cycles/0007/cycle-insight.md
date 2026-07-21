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
