# Cross-Cycle Insights

## Recurring wins

- **Two-segment SLRU (80/20 protected/probationary) is the load-bearing foundation.** Every winning variant since cycle 1 rests on this structure; single-segment LRU is uniformly worse across all tested policy axes.
- **Symmetric signal application at both eviction sites.** Applying the eviction signal (frequency or IRR) only in the protected segment while leaving probationary as blind LRU always left performance on the table; closing the asymmetry (cycle 4) was a consistent gain that held when the signal changed to IRR (cycle 8).
- **Signal preservation across segment transitions.** Counter halving on demotion (frequency era) and timestamp preservation across demotion (IRR era) both proved load-bearing; resetting or discarding the signal at the segment boundary degraded quality in every test.
- **Probationary half-scan is consistently necessary.** Eviction-worthy entries are distributed through probationary, not compressed near the LRU head — confirmed under both the frequency baseline (cycle 5) and the IRR baseline (cycle 8). The equivalent quarter-scan suffices for protected under both policy regimes.
- **IRR timestamps produced the largest single-cycle gain in the series (cycle 7).** Switching from cumulative access counts to inter-reference recency broke out of a three-cycle neutral-winner plateau, confirming that recency-of-recency is a sharper eviction signal than total frequency on this workload.

## Recurring failures

- **Ghost admission is exhausted.** Tested at probationary capacity (cycle 2 — gain was SLRU, not ghost), isolated on the frequency baseline (cycle 3 — negligible), at full capacity (cycle 4 — regressed), at half-total capacity (cycle 6 — regressed), and with IRR (cycle 8 — regressed). The reuse distances on this workload do not produce a reliable ghost-intercept signal at any window size tested.
- **Widening probationary beyond 20% always regresses.** Confirmed independently on the frequency baseline (cycle 2) and the IRR baseline (cycle 8). The active working set is dense enough that an 80% protected segment is a hard floor; any larger probationary triggers excess demotion churn.
- **Fixed-rate counter decay collapses the eviction signal.** A fixed logical-clock half-life (cycle 6) erased the distinction between demoted-warm and cold-new entries, the exact distinction the scan depends on. The right decay constant is workload-specific; no fixed value is likely to generalize.
- **Scan-fraction narrowing in probationary always fails.** Reducing from half to quarter in probationary degraded quality under both frequency (cycle 5) and IRR (cycle 8) baselines — IRR variation is not compressed near the LRU head.
- **Second-chance reprieve bits are inert or harmful.** Freezing node position on protected hits destroyed recency ordering (cycle 3). When the position-freeze was removed (cycle 6), the referenced-bit tie-breaker became harmless but also inert — equal-frequency collisions targeted by the mechanism do not occur at the current scan window width on this trace.
- **Scan-fraction widening in protected yields diminishing returns.** Quarter-to-third expansion in protected (cycle 4) produced marginal movement; going further is not a productive direction on this workload.

## Open questions

- **Hybrid IRR-plus-visit-count signal.** Pure IRR discards long-term warmth history that frequency-halving preserved across demotion cycles; a coarse visit counter layered onto IRR might recover that without reintroducing staleness from unbounded accumulation.
- **Multi-workload evaluation is overdue.** All wins and failures are anchored to a single LIRS-generated trace. It is unknown whether the IRR gain is broadly transferable or specifically aligned to this trace's reuse-distance distribution. This is the highest-value open question.
- **Adaptive segment-ratio balancing (H01).** Never evaluated despite being nominated in cycles 7 and 8. With a sharper eviction signal in place, the optimal protected/probationary ratio might differ from the inherited 80/20; live hit-rate feedback could drive the split dynamically.
- **Adaptive scan fractions.** Fixed fractions (quarter, half) have been validated empirically but not tuned to workload shape. Fractions proportional to observed reuse-distance distributions might yield equivalent eviction quality at lower scan overhead, especially on the IRR baseline where the signal is sharper.

## Per-policy notes

- **Baseline LRU:** scan-pollution vulnerability confirmed cycle 1 and never recovered; all variants across all cycles improved over it. Serves only as a reference floor.
- **CLOCK:** outperformed by SLRU in cycle 1; avoids linked-list pointer maintenance but acts at eviction time, a weaker intervention point than SLRU's insertion-time routing. Not revisited.
- **SLRU + frequency-aware eviction (cycles 3–6):** rescued from an early design failure (too-small scan window, counter reset on demotion) in cycle 3; peak form is two-segment 80/20, halving-on-demotion, quarter-scan in protected, half-scan in probationary, MAX_FREQ cap. Hit a local optimum by cycle 5; three consecutive neutral-winner cycles confirmed it.
- **SLRU + IRR timestamps (cycles 7–8):** current frontier baseline; replaces halving-on-demotion with timestamp preservation naturally, no ghost set. Scan fractions and segment ratio confirmed at the same values as the frequency baseline. Ghost admission incompatible. Plateau not yet confirmed — H01 and multi-workload evaluation remain.
- **Ghost admission (all cycles):** never produced an isolated win; effect is always negative or negligible when separated from the structural SLRU gain. Fully mapped across the meaningful window range on two underlying policies.
