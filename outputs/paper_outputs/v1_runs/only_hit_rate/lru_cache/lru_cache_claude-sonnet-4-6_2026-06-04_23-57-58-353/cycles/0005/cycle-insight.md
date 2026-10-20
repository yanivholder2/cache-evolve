# Cycle 0005 Insight

## Cohort outcome

All three variants probed the cycle-4 frequency-aware baseline from different angles — scan-fraction narrowing, counter inflation control, and admission gating — and none produced a hit-rate gain. The winner (v002) advanced on neutrality: the frequency cap introduced no regression while the other two variants hurt performance, making it the best performer in a cohort that collectively confirmed the current configuration is near a local optimum on this workload.

## Patterns

- The probationary half-scan is now confirmed as genuinely necessary, not merely conservative. Frequency variation among probationary entries — cold newcomers mixed with demoted entries carrying halved counters spanning a wider range — is spread throughout the list, not compressed near the LRU head. The quarter-scan that suffices for protected (where all entries have at least one access and frequency variation near the LRU end is compressed) does not transfer to probationary.
- Counter inflation is not an active problem on this workload. The halving-on-demotion mechanism already prevents runaway accumulation in practice: entries must remain in protected without ever being demoted for counters to inflate far above the cap, and that residency pattern does not occur at this cache size on workload-a. The MAX_FREQ=8 cap is structurally sound as a guardrail but targets a pathology the trace does not exhibit.
- The frequency-aware eviction scan introduced in cycle 4 already provides sufficient cold-traffic filtering in probationary. A two-hit promotion gate is redundant protection — the entries that survive in probationary are generally worth promoting immediately, and the extra exposure window costs more hot-key residency than the one-hit-wonder reduction saves.
- Two consecutive cohorts (cycles 4 and 5) have produced wins that are structural closures rather than fresh gains: cycle 4 closed the asymmetry between eviction sites; cycle 5 confirmed that three further refinements of the resulting design produce no improvement. This is a convergence signal for the current policy on this workload.

## Open questions

- Is the policy now at a local optimum for workload-a, or are there unexplored structural directions (e.g., adaptive scan fractions tied to observed hit-rate feedback, segment-ratio tuning, or load-sensitive demotion) that could break out of the current plateau?
- Would the frequency cap (v002's change) matter on a workload with longer-lived hotspots where continuous protected residency without demotion is more common?
- Is widening the evaluation to a second workload the most informative next step — to distinguish genuine convergence from workload-specific saturation?

## Continuation

Next cycle should carry v002 forward as the new baseline (the MAX_FREQ=8 cap is a harmless guardrail worth keeping). The probationary half-scan and immediate single-hit promotion are both settled — neither narrowing the scan nor adding a promotion gate is productive on this workload. With three scan-fraction and admission variants now exhausted, the highest-value direction is either a genuinely new structural axis (adaptive policy parameters, segment-ratio exploration beyond the previously tested range, or a different eviction philosophy) or explicit multi-workload evaluation to determine whether the policy has converged broadly or only on workload-a.
