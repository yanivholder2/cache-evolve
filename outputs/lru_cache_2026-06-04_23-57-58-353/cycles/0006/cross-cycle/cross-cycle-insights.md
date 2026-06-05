## Recurring wins

- Two-segment SLRU is the foundational structural win: routing new entries into a probationary buffer before they can displace the protected working set neutralises scan pollution at insertion time, which is stronger than acting only at eviction time.
- Frequency-aware eviction scanning a fixed fraction of each segment's LRU tail reliably outperforms blind LRU eviction; the mechanism requires a large enough scan window to create real selection pressure (a constant small count fails; a segment-proportional fraction succeeds).
- Counter halving on demotion is load-bearing: it preserves the frequency history accumulated in protected so that demoted entries remain distinguishable from cold newcomers at the probationary eviction site. Without it, the frequency signal evaporates across segment transitions.
- Applying frequency-aware eviction symmetrically at both eviction sites (protected demotion and probationary eviction) eliminated the largest remaining structural gap after cycles 1-3; asymmetric application left gains on the table.
- The MAX_FREQ cap is a harmless, structurally sound guardrail: it introduced no regression on this workload and protects against counter inflation on traces where protected residency without demotion is prolonged.

## Recurring failures

- Ghost admission has been tested at three window sizes (probationary capacity, half total capacity, full capacity) and regressed or was negligible each time. Reuse events at distances beyond the probationary segment are cold traffic on this workload, not deferred-warm; no window sizing rescues the mechanism here.
- Scan-fraction tuning above the established fractions (one-quarter for protected, one-half for probationary) yields negligible returns; frequency variation near the LRU tail is already captured by the narrower windows.
- Frequency-aware demotion with a fixed decay constant (logical-clock half-life) collapses the very distinction it was meant to sharpen: any single constant either decays too fast (destroying the protected-vs-newcomer signal) or too slow (equivalent to no decay). The idea is acutely workload-sensitive and requires reuse-distance calibration to be viable.
- Second-chance reprieve bits that suppress move-to-tail on protected hits degrade the recency ordering and ultimately strand the entries they were meant to protect; position-update suppression is not compensated by reprieve logic alone.
- Two consecutive cohorts (cycles 5 and 6) have produced winners that advanced on neutrality rather than gain, signalling that all major structural gaps in the current policy have been closed on workload-a.

## Open questions

- Is the policy at a local optimum on workload-a, or do unexplored structural philosophies — adaptive segment-ratio balancing (ARC-style feedback), load-sensitive demotion, or admission filters — offer a path to further gain?
- Would the frequency cap, decay, or referenced-bit tie-breaker matter on a workload with longer-lived hotspots or a different reuse-distance distribution? These mechanisms are inert on workload-a but may be valuable elsewhere.
- Multi-workload evaluation is now the clearest next step: is the accumulated policy broadly strong, or specifically tuned to the scan-heavy profile of workload-a?
- Could frequency-aware demotion with a decay constant calibrated to the observed inter-access gap distribution (rather than a fixed half-life) recover the cycle-6 failure, or is any fixed constant wrong on a non-stationary trace?
- The referenced-bit tie-breaker is structurally correct but inactive at the current scan window; would a wider scan window expose enough equal-frequency collisions to make it measurably useful?

## Per-policy notes

- **Plain LRU (baseline):** Confirmed vulnerable to scan pollution; cold-traffic bursts displace warm entries because one-hit and multi-hit entries are indistinguishable. All subsequent policies improve on it.
- **CLOCK (cycle 1, v002):** Eliminates linked-list pointer maintenance; acts only at eviction time, which is a weaker signal than insertion-time routing. Lost to SLRU in the initial cohort but may offer throughput benefits under high concurrency not captured by hit-rate measurement alone.
- **Frequency-window eviction (cycle 1, v003):** Bounded by a fixed small look-ahead count; failed at constant size, succeeded when scaled to a segment-proportional fraction (cycle 3). The eviction-time placement is not a dead end — the cycle-1 failure was a window-size design flaw.
- **SLRU (cycle 1, v001 onward):** Foundation of all winning designs from cycle 1 forward; 80/20 protected-to-probationary split confirmed better than 70/30 (cycle 2); larger split toward probationary triggers demotion churn because the hot working set exceeds the smaller protected segment.
- **Frequency-aware demotion (cycle 3, v002 onward):** Required two fixes over its initial failure — proportional scan window and counter halving rather than reset. Once repaired, it has been the advancing winner for four consecutive cycles.
- **Ghost admission (cycles 2-4, 6):** Marginal or negative across every window size tested; adds bookkeeping overhead for no measurable gain on this workload.
- **Promotion gating / two-hit requirement (cycle 5, v003):** Redundant given frequency-aware eviction already filters cold probationary entries; costs hot-key residency without reducing one-hit-wonder promotions in practice.
- **Referenced-bit tie-breaker (cycle 6, v001):** Structurally sound and avoids the cycle-3 position-freeze error; inert on this workload because equal-frequency collision at the narrow scan window is uncommon on this trace.
- **Logical-clock decay (cycle 6, v003):** Workload-specific failure; collapsed the frequency signal rather than sharpening it. Not viable with a fixed half-life on this trace.
