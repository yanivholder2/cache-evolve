# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0007

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants probed retention depth and admission quality: raising the second-chance frequency ceiling (v001), widening small-queue admission (v002), and granting ghost-re-admitted items a head-start credit (v003). Only the ceiling raise won, and the gain was real but modest.

## Patterns

- **MAX_FREQ = 4 yields a genuine but incremental gain.** A non-trivial minority of items reach the old ceiling and benefit from the extra retention pass, confirming the upward direction is viable — but the small magnitude of the improvement suggests this minority is limited, and proportional returns from further ceiling increases are unlikely.
- **The freq > 0 admission gate is definitively closed.** Tried twice (neutral solo in cycle 0003, harmful here in combination with ghost precision), it never helps. Diluting main with single-access items costs more in churn than the marginal ghost-hit promotions can recover; the two mechanisms are orthogonal rather than synergistic on this workload.
- **Ghost re-admission credits are counterproductive.** Seeding re-admitted items at freq=1 displaces incumbents that earned their slots through genuine accesses. The ghost signal confirms re-use potential, but not enough to justify claiming a residency advantage over established main-queue residents.

## Open questions

- Is there a useful stopping point between MAX_FREQ = 4 and a higher ceiling (e.g., 5 or 6), or does the frequency distribution of hot items make any further step negligible?
- With admission thresholds and ghost sizing now well-characterised, are there structural axes entirely unexplored — such as eviction ordering within main, hit-count handling on ghost re-admission at freq=0, or adaptive queue sizing under phase changes?

## Continuation

Next cycle should adopt MAX_FREQ = 4 as the new baseline and make one cautious test of MAX_FREQ = 5, but with low expectations given the already-small delta from 3 → 4. The admission threshold and ghost credit directions are exhausted and should not be revisited. If the MAX_FREQ = 5 probe is also incremental or flat, the highest-yield path forward is likely a genuinely new structural axis — the queue-sizing and admission-threshold knobs have been mapped to their effective limits.


### Cycle 0005

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


### Cycle 0006

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
