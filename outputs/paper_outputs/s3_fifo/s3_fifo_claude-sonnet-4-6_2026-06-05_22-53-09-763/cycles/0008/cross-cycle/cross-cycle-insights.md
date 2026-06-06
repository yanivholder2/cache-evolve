# Cross-Cycle Insights

## Recurring wins

- **Shrinking the small queue transfers slots to main and consistently improves resident quality.** The benefit held from 25% → 10% (cycle 1), continued to ~8% (cycle 2), and remained non-exhausted at ~7% (cycle 8); the effective floor has not yet been reached.
- **Ghost precision at roughly main_size/2 is a reliable positive.** Halving the ghost cap improved hit rate in cycles 1–2 regardless of small-queue ratio; this cap has been confirmed as both a beneficial tightening and a hard floor (further compression hurts in cycle 3).
- **Raising MAX_FREQ ceiling yields genuine but diminishing gains.** Each increment (3→4, 4→5) improved hit rate; the improvements are real, confirmed across cycles 4–5, but shrink monotonically with each step.
- **Unconditional move-to-front on main hits is a confirmed small positive** (cycle 8). Proactive recency refresh prevents tail drift; the gain is incremental but reliable.

## Recurring failures

- **Relaxing the admission threshold (freq > 0)** was tried in cycles 1, 3, and 4 — neutral at best, harmful in combination. The narrow small queue starves single-access traffic before it can benefit; this axis is closed.
- **Expanding the ghost window** beyond the baseline cap introduced noise from distant, non-recurring accesses and hurt hit rate (cycle 1). Ghost value is precision, not coverage.
- **Ghost re-admission credits** (seeding re-admitted items at freq=1) are counterproductive — they displace incumbents with genuine access histories rather than giving ghost-validated items a legitimate advantage (cycle 4).
- **Positional manipulation within main** (saturation-triggered head-repositioning, midpoint insertion) is inert; hot items that reach the frequency ceiling rarely sit near the eviction frontier, and reactive recycling already rescues them (cycle 6).
- **Adaptive/phase-sensitive mechanisms fail on workload-a**, which is phase-stable (Zipfian, no scan bursts). Controllers fire on noise, not real phase shifts, causing slight regressions (cycles 7–8). These mechanisms are workload-suitability failures, not design failures.
- **Cross-feature threshold transfer is unsafe.** A second-chance threshold that worked before eager promotion was introduced (cycle 7) became harmful afterward, because eager promotion deposits items into main at freq==2, and the tighter gate then evicts those fresh hot items on their first pass (cycle 8).

## Open questions

- Does small-queue shrinkage continue to pay off below ~7%, or is there a floor where the filter becomes too coarse and cold items begin polluting main?
- Can a frequency-relative second-chance threshold (rather than an absolute cutoff) capture the original stale-squatter intent without penalising recently-promoted items whose entry frequency reflects eager promotion rather than genuine warmth?
- Would the adaptive threshold mechanism perform as hypothesised on a workload with genuine phase transitions or scan bursts — contexts where the control signal would carry real information?
- How much of the remaining performance gap separates the current configuration from an alternative algorithm (e.g., TinyLFU, SIEVE) on workload-a? Determining this would clarify whether further S3-FIFO refinement is worth pursuing at all.

## Per-policy notes

- **S3-FIFO — small queue:** Monotone benefit from capacity reduction across every cycle that tested it; the earlier conclusion that 10% was near-optimal was an untested assumption. Further probing below ~7% is the highest-remaining-yield direction within the queue-sizing axis.
- **S3-FIFO — main queue:** Move-to-front during eviction is the primary recency-sorting mechanism, not merely a second-chance gate. Removing it (cycle 5, CLOCK-style scan) caused the largest regression observed across all cycles. Any eviction-ordering experiment must preserve or carefully augment this property.
- **S3-FIFO — ghost queue:** main_size/2 is an empirically confirmed two-sided boundary: expanding hurts precision (cycle 1), compressing below it loses genuine re-use signal (cycle 3). Ghost cap should stay fixed at this value.
- **S3-FIFO — frequency ceiling (MAX_FREQ):** The marginal return per increment is strictly declining. The 3→4 step was modest; 4→5 was smaller still. Further increments are not a productive search direction.
- **S3-FIFO — overall convergence:** After eight cycles, all major axes within this algorithm on workload-a — queue sizing, admission thresholds, ghost sizing, frequency ceiling, ghost credits, eviction ordering, positional tuning, promotion timing, and tournament selection — have been mapped to diminishing returns or confirmed failures. The configuration is near-optimal for this workload; structural novelty or an algorithm comparison is the only remaining high-yield direction.
