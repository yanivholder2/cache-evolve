## Recurring wins

- **Two-segment SLRU** has been the load-bearing foundation since cycle 1; acting at insertion time (routing every new entry into probationary before it can displace the protected working set) reliably outperforms eviction-time-only policies on scan-heavy workloads.
- **Counter halving on demotion** (rather than resetting) is essential for frequency-aware demotion to work: it lets accumulated access history survive segment transitions so recently-warm entries remain distinguishable from long-tenured ones.
- **Scan fractions proportional to segment size** consistently outperform fixed small counts; a fraction-based scan creates genuine selection pressure whereas a constant small window leaves most of the frequency signal unexploited.
- **Symmetric frequency-aware eviction** — applied at both eviction sites (protected demotion and probationary eviction) — closed a structural asymmetry that had been silently discarding the frequency signal at the probationary boundary.
- **80/20 protected-to-probationary ratio** has been stress-tested from both sides (cycle 1 vs. plain LRU; cycle 2 vs. 70/30) and confirmed as the right split for this workload's working-set size.

## Recurring failures

- **Ghost admission** was tested in isolation in cycle 3 after cycle 2 ambiguity; it contributes negligibly on this workload. The ghost window bounded to probationary capacity cannot intercept the reuse-distance gap typical of this trace, and expanding it to full capacity (cycle 4) actively regresses hit rate by admitting unreliable locality signals.
- **Eviction-time-only frequency policies** (fixed small window, constant scan count) keep failing when applied without counter persistence and proportional window sizing; the signal evaporates before it accumulates durable discrimination.
- **Counter reset on demotion** kills frequency-aware demotion regardless of scan-window size; the cycle-2 failure was design-flaw-driven, not a fundamental ceiling on frequency-based approaches.
- **Second-chance reprieve bits that suppress position updates** degrade protected-segment ordering; the continuous move-to-tail on access is load-bearing — any mechanism that freezes node position on hits strands hot entries near the demotion head.
- **Two-hit promotion gating** is redundant when frequency-aware eviction already filters probationary entries; the extra exposure window costs more hot-key residency than the one-hit-wonder reduction saves (cycle 5).
- **Widening the protected scan fraction beyond 1/4** yields diminishing returns: frequency variation near the LRU end of protected is already captured by the narrower window, and broader scans add overhead without new signal.

## Open questions

- **Workload generality**: five cycles confirm convergence on workload-a, but whether this is a local optimum for this specific trace or a broadly robust policy is unknown; multi-workload evaluation is the clearest way to distinguish the two.
- **Adaptive scan fractions**: both scan fractions (1/4 for protected, 1/2 for probationary) are currently fixed; whether feedback-driven adaptation tied to observed hit rate or reuse-distance distribution could break the current plateau is unexplored.
- **MAX_FREQ cap at longer-lived hotspots**: the cycle-5 frequency cap is inert on this workload because halving-on-demotion already controls counter growth; a workload with sustained protected residency and no demotion events might expose the cap as a meaningful constraint.
- **Salvageable second-chance**: the cycle-3 failure was the position-freeze trade-off, not reprieve logic per se; a variant that sets the referenced bit while still performing move-to-tail might recover the reprieve benefit without degrading recency ordering.
- **Intermediate ghost window**: the only tested points are probationary capacity (negligible effect) and full capacity (regression); a window at ~50% of total capacity has not been evaluated and might capture cross-residency warmth without the admission-quality problem.

## Per-policy notes

- **Plain LRU (baseline)**: scan-pollution vulnerability is real and confirmed addressable by multiple mechanisms; no variant in any cycle has regressed below it.
- **CLOCK (cycle 1, v002)**: eliminated linked-list pointer maintenance but acts only at eviction time; the concurrency and throughput implications under high load have never been measured and remain an open structural question.
- **Two-segment SLRU**: the 80/20 split is settled; both narrower (30% probationary) and looser (unchanged from cycle 1) directions have been tested; segment-ratio tuning is a diminishing-return axis.
- **Frequency-aware demotion in protected** (cycle 3+): LRU-quarter scan with counter halving is settled and effective; widening the fraction to 1/3 bought nothing (cycle 4).
- **Frequency-aware eviction in probationary** (cycle 4+): LRU-half scan is settled; narrowing to 1/4 regressed (cycle 5), confirming the probationary counter distribution is wider and requires the broader window.
- **Ghost admission**: negligible on this workload across two clean tests (cycles 2 and 3); deprioritized.
- **MAX_FREQ=8 cap** (cycle 5 forward): structurally sound guardrail, currently inactive on this trace; kept as a harmless safety measure.
