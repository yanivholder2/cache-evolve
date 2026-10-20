# Cycle 0006 Insight

## Cohort outcome

All three variants probed fine-grained positional and threshold adjustments within main — tightening the second-chance gate, repositioning saturated items proactively, and varying the insertion point of newly promoted items — and produced the smallest aggregate movement of any cycle to date: two negligible positives and one true zero.

## Patterns

- **Positional manipulation within main is inert on this workload.** Both the saturation-triggered head-repositioning (v002) and the midpoint-insertion variant (v003) found that the hot set almost never satisfies the combination of conditions required to benefit: items that reach the frequency ceiling rarely sit close enough to the eviction tail for a position rescue to matter, and the eviction scan recycles them adequately without physical relocation. Fine-grained placement is not a lever.
- **The freq=1 population is too small and cold to be worth gate-keeping.** Tightening the second-chance threshold moved the metric in the predicted direction, but the magnitude confirms that freq=1 traffic is a negligible fraction of re-access candidates on this workload. The hypothesis was correct in principle but irrelevant in practice.
- **The configuration is approaching a local optimum.** The cohort's combined effect on hit-rate is barely measurable. All axes probed across six cycles — ghost sizing, admission thresholds, frequency ceiling, ghost credits, eviction ordering, and now positional tuning — have been mapped to diminishing returns or outright failure.

## Open questions

- Is the current configuration genuinely near-optimal for workload-a, or is there a structural axis that hasn't been framed yet — such as adaptive queue sizing under phase changes, scan-burst detection, or workload-specific eviction tie-breaking?
- Given that move-to-front is the dominant recency mechanism and positional refinements don't help, could *conditional* move-to-front (e.g., only reposition above a frequency threshold) extract any remaining signal without regressing to the cycle-0005 failure mode?

## Continuation

The static-parameter and positional-tuning search spaces are exhausted; further incremental adjustments to thresholds, insertion points, or ceiling values are unlikely to yield meaningful gains. The next cycle's hypotheses-writer should either commit to a genuinely new structural axis — adaptive or workload-phase-sensitive mechanisms are the most plausible candidates — or explicitly declare the configuration converged and benchmark it against an alternative algorithm rather than continuing to refine knobs within the current design.
