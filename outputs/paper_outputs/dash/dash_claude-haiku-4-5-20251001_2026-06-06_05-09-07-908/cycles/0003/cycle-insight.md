# Cycle 0003 Insight

## Cohort outcome

This cycle tested composition of geometry and frequency gating, plus further geometry concentration. Two strategies succeeded: frequency-gated promotion combined with aggressive (62.5/37.5) geometry, and pushing geometry even further to 75/25 allocation. The third variant (frequency gating on balanced 50/50 geometry) revealed that the gate's threshold interacts poorly with less-concentrated geometry. The winning variants confirm that geometry remains the primary performance lever, while frequency gating amplifies its benefits only on appropriately tuned configurations.

## Patterns

- **Frequency-gating success depends on underlying geometry**: v002's gating achieved both latency and hit-rate wins on 62.5/37.5 geometry, but v001's identical gate hurt hit-rate on 50/50 geometry. The threshold of 2 is too conservative for balanced geometry (filters too many promotion candidates) but well-calibrated for more concentrated geometry (filters mostly false-positives from the smaller stash tier). This suggests the gate's selectivity must match the geometry's candidacy rate.

- **Geometry asymptote continues to shift upward**: The progression from baseline (25/75) through 50/50, 62.5/37.5, to 75/25 shows sustained improvement across both hit-rate and latency. The cycle-0002 hypothesis that 62.5/37.5 might be saturating was incorrect; 75/25 delivers cleaner wins on both dimensions without apparent regression, suggesting the concentration axis has further room.

- **Composition behavior is non-linear across parameter space**: While geometry and frequency gating were mechanically expected to compose orthogonally, the outcome varies sharply with their specific values. This is a departure from cycle-0001's clean stacking and cycle-0002's earlier synergies, indicating the optimization landscape has entered a region where parameter interactions matter as much as axis independence.

- **Simple geometry beats complex layering when both win**: v003 improves hit-rate, get latency, and put latency simultaneously using only geometry reallocation—no additional promotion logic. v002 achieves similar gains using gating + geometry. When both strategies succeed, the simpler one (v003) suggests the workload's optimization margin is primarily unlocked by capacity allocation, not promotion filtering.

## Open questions

- Does frequency gating (threshold 2) on v003's 75/25 geometry compose without regression, or will the gate again filter too few false-positives to justify overhead?
- Is 75/25 the asymptote, or does further geometry concentration (e.g., 85/15, 90/10) continue to improve both metrics?
- Would an adaptive frequency threshold (scaled to geometry allocation ratio) eliminate the composition sensitivity and make gating orthogonal across all geometry settings?

## Continuation

Geometry concentration has proven to be the consistent performance lever across all three cycles, with no saturation observed yet. Next cycle should test whether frequency gating works on v003's 75/25 geometry and explore the asymptote question by pushing geometry even more aggressively. Consider whether the frequency threshold should scale adaptively with geometry concentration (e.g., threshold = max(2, floor(stash_buckets / 4))) to stabilize gating's selectivity across different allocations. If adaptive thresholds prove effective, they could unlock clean composition across the parameter space; if not, prefer geometry-only variants as they deliver reliable wins with less tuning burden.
