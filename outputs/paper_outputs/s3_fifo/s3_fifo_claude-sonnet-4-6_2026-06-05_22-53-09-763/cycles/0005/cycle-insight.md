# Cycle 0005 Insight

## Cohort outcome

This cycle probed three independent axes — frequency-ceiling extension, eviction-ordering strategy within main, and promoted-item frequency normalization — yielding one tiny positive (v001), one significant regression (v002), and one neutral result (v003). The winner advances the configuration by the smallest margin seen across all cycles.

## Patterns

- **The frequency-ceiling axis is now exhausted.** The 4→5 increment produced an even smaller gain than 3→4, and the v001 insight characterises the hot-set tail as sparse above the old ceiling. Each step yields a shrinking return; further increments are not a productive direction.
- **Move-to-front reprieve in main eviction is load-bearing recency sorting, not merely a second-chance gate.** The CLOCK-style in-place scan tried in v002 revealed that the baseline's positional churn continuously keeps recently re-accessed items away from the eviction frontier. Removing that implicit recency re-ordering — even while preserving the frequency check — caused a large regression. Eviction ordering is a first-class dimension of this cache's performance.
- **Promoted-item frequency normalization is inert on this workload.** Capping incoming main-queue items at freq=1 had no measurable effect, indicating that over-credited promotions are not displacing established hot residents in practice. This axis can be set aside.

## Open questions

- Since move-to-front is the primary recency-sorting mechanism, are there modifications to that behaviour (e.g., conditional repositioning above a frequency threshold, or partial repositioning) that could improve hit rate further without discarding the recency benefit?
- With queue sizing, admission thresholds, ghost sizing, frequency ceiling, and promotion normalization all characterised, what genuinely unexplored structural axes remain? Candidates include adaptive queue sizing under workload phase shifts, eviction tie-breaking within frequency bands, or behaviour specific to scan-heavy versus hot-set-heavy access phases.

## Continuation

Both the frequency-ceiling and promotion-normalization axes are now closed. The most actionable finding from this cycle is that eviction ordering within main is the most sensitive structural lever yet identified — the implicit recency sort embedded in move-to-front is the real mechanism behind the baseline's performance. The next cycle should treat this as the primary axis: explore whether selective or conditional repositioning during eviction can improve on the baseline's unconditional move-to-front, rather than continuing to chase diminishing returns on the well-mapped parameter knobs.
