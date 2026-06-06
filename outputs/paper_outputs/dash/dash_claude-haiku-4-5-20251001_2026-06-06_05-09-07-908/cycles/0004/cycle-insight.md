# Cycle 0004 Insight

## Cohort outcome

This cycle tested three strategies on an increasingly concentrated bucket geometry: frequency-gated promotion on 75/25 geometry, an even more aggressive 85/15 geometry shift, and back-insert promotion composition. All three variants succeeded in at least one metric. Geometry extension to 85/15 delivered clean wins across both hit-rate and latency; frequency-gating improved hit-rate but incurred a latency cost; back-insert achieved the same hit-rate as its parent but with latency regression. The clear standout is the pure-geometry variant, confirming that capacity reallocation remains the dominant performance lever.

## Patterns

- **Geometry asymptote remains unblocked**: The progression from baseline (25/75) through 75/25 to 85/15 shows sustained improvement in both hit-rate and latency, with the latest geometry delivering 7.3-point hit-rate gains and meaningful latency improvements across all percentiles. The prior cycle's hypothesis that geometry might be saturating was again disproven.

- **Composition complexity continues to diverge from mechanical independence**: While frequency-gating (v001) and back-insert (v003) are logically orthogonal to geometry, their empirical outcomes differed sharply. Gating improved hit-rate (confirming cycle 0003's selectivity scales to tighter allocations) but introduced measurable get-latency overhead. Back-insert—despite composition with the most aggressive normal-tier allocation yet—failed to improve any metric and regressed latency on both get and put, suggesting the MRU-insertion cost is truly fundamental and cannot be absorbed by larger primary-tier capacity.

- **Simple variants outperform when all win**: v002 (pure geometry 85/15) achieved the cleanest wins across both dimensions—no trade-offs, no latency penalty. When multiple variants succeed, the mechanically simpler approach consistently delivers better overall outcomes. This aligns with cycle 0003's observation that geometry-only variants are more reliable than composition-based strategies.

- **Frequency-gating selectivity scales well to concentrated geometry**: v001's successful hit-rate improvement confirms that the counter ≥ 2 threshold remains well-calibrated even on 75/25 (and now, implicitly, 85/15-adjacent) geometry, filtering mostly false-positives in a shrinking stash tier. The trade-off suggests hit-accuracy and latency form a frontier under this parameter regime.

- **Back-insert's overhead is structural, not capacity-dependent**: v003's consistent latency regressions on both get and put operations, despite being paired with the most generous hash-indexed allocation tested, indicate the MRU-insertion mechanism carries an inherent cost that no amount of primary-tier capacity can overcome.

## Open questions

- Is 85/15 the asymptote, or do even more aggressive allocations (90/10, 95/5) continue to improve both metrics without saturation?
- Could frequency-gating's latency penalty be reduced by tuning the threshold downward (e.g., threshold = 1) without losing hit-rate selectivity on concentrated geometry?
- Why does back-insert fail so consistently across all tested geometries (50/50, 62.5/37.5, 75/25, 85/15), and are there alternative promotion-insertion strategies that avoid this overhead?

## Continuation

Geometry concentration has proven to be the most reliable and composable optimization axis across all four cycles, with no saturation observed and no apparent downside. Next cycle should push geometry even further (e.g., 90/10 or beyond) to identify the true asymptote and understand whether diminishing returns eventually appear. Back-insert should be dropped as a composition strategy—it is fundamentally costly and incompatible with this workload across all tested configurations. If pursuing frequency-gating refinement, focus on threshold tuning to minimize latency impact while preserving hit-rate gains. Otherwise, geometry-only variants remain the safest bet for consistent, clean improvements. Consider whether entirely different optimization axes (beyond promotion logic and bucket allocation) might unlock further gains on top of aggressive geometry.
