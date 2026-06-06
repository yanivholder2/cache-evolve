# Cycle 0007 Insight

## Cohort outcome

This cycle tested three distinct approaches to advance beyond cycle 0006's latency breakthrough: adding ghost-frequency decay to a lean baseline (v001), replacing the fixed ghost re-admission boost with occupancy-responsive scaling (v002), and using recency instead of frequency as the ghost re-admission signal (v003). Only v002 delivered clear dual-metric wins—v001 reintroduced per-operation overhead without recovering hit-rate, while v003 achieved dramatic latency gains at the cost of hit-rate loss. The cohort confirms that adaptive re-admission signals outperform both fixed formulas and frequency-decay approaches when paired with the right architectural foundation.

## Patterns

- **Occupancy-responsive adaptive boosting outperforms fixed-boost policies**: v002 proves that scaling ghost re-admission strength with main-queue occupancy delivers both latency improvement and hit-rate stability, extending cycle 0003's success with occupancy-aware thresholds into the realm of signal adaptation. The variable boost eliminates thrashing that fixed +3 formulas trigger during capacity transitions.

- **Recency is computationally efficient but predictively weak**: v003 demonstrates that entry position in ghost requires cheaper arithmetic than frequency lookup, yielding 43–57% latency improvements. However, recency alone is a poor proxy for re-access likelihood in this workload—temporal distance from eviction does not capture the same patterns that frequency or occupancy-based signals do, resulting in hit-rate loss.

- **Ghost-frequency decay cannot substitute for structural adaptation on lean baselines**: v001's attempt to add frequency preservation (-3 depth) to cycle 0005/v002's lean approach failed on both fronts: the per-operation overhead of ghost entry search and boosting contradicted the lean design, while the passive recovery mechanism could not replace the active barrier adaptation that structural sizing provides.

- **Pre-computed ghost-queue state with stored metadata remains the durable architectural choice**: All three variants leveraged ghost entries containing frequency or state information, confirming cycle 0006/v002's architectural insight that pre-computation enables efficiency. The critical variable is the re-admission signal used to exploit that state.

## Open questions

- Does combining v002's occupancy-responsive formula with recency-based filtering (e.g., prioritizing re-admission of recently-evicted items) recover both v002's latency and v003's hit-rate potential?

- Can v003's recency-to-boost scaling coefficients be tuned (e.g., 2–5 range instead of 2–4) to recover the hit-rate loss while retaining latency gains, or is the predictive gap structural?

- Is there a sweet spot for ghost-frequency decay depth (between -1 and -3) that reduces per-operation overhead while preserving enough warm candidates to improve hit-rate on lean baselines?

## Continuation

The cohort validates that **adaptive re-admission signals scale with workload state better than fixed formulas**, making v002 the strongest baseline for balanced performance. Recency (v003) opens a new latency frontier but at measurable hit-rate cost; the next cycle should explore whether signal combination or tuning recovers both gains. Ghost-frequency decay on lean architectures (v001) consistently reintroduces overhead costs that offset any hit-rate benefit—this direction should be deprioritized unless paired with structural changes that justify the per-operation cost. **For the next cycle:** (1) integrate v002's occupancy-responsive boost as the new baseline if latency-hit-rate balance is the goal; (2) if latency specialization is acceptable, v003's recency approach is near-optimal and worth tuning for that use case; (3) explore hybrid re-admission signals (frequency + recency, occupancy + recency) to test whether combining signals recovers v003's latency while protecting hit-rate; (4) avoid further ghost-frequency-decay variants unless they couple to structural mechanisms (e.g., dynamic sizing, adaptive thresholds) that actively use the preserved state.
