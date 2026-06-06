# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three variants tested different angles on the admission gate: sketch sizing, frequency decay, and gate threshold. Only the sketch-width increase (v001) improved hit rate; attempts to refresh frequency signal via decay (v002) or relax the gate threshold (v003) both degraded it.

## Patterns

- **Precision beats permissiveness**: The winner (wider sketch) improved frequency estimation fidelity while keeping admission logic unchanged. Both hurt variants tried to change the logic itself—either by decaying frequencies or accepting tied candidates—assuming the baseline was too strict. The data suggests the baseline's strictness is not the problem; measurement noise is.

- **Sketch width is a practical bottleneck**: Hash collisions in the Count-Min sketch directly degrade admission precision. Doubling the width factor (4→8) reduced collision errors and improved hit rate, confirming that sketch sizing, not responsiveness or recency effects, was limiting performance.

- **Early frequency signal is valuable**: v002's hypothesis that periodic decay would favor recent patterns over stale ones backfired—halving counters every capacity puts removes signal too aggressively. Items that earned high frequencies early are genuinely predictive, and monotonic accumulation preserves that signal better than periodic resets.

- **Ambiguity favors residents**: v003's relaxation from strict `>` to `>=` allowed tied frequency estimates to evict hot residents. When the sketch produces equal frequency estimates, accepting the tie admits marginal candidates based on recency alone, which harms hit rate. The strict gate's selectivity protects hot items during uncertainty.

## Open questions

- Does hit rate continue to improve with even wider sketches, or is there a diminishing return above 8x?
- Are other sketch parameters (depth, counter saturation strategy) worth tuning alongside width?
- Is the baseline's monotonic counter saturation at u32::MAX itself a limiting factor, or is width the only knob needed?

## Continuation

Pursue wider sketch sizing as the primary tuning axis—the baseline sketch was genuinely undersized. Drop hypothesis categories around decay, recency-weighting, and threshold relaxation; the admission logic is sound. For the next cycle, explore whether sketch depth matters, whether saturation saturation handling can improve further, or whether alternative frequency-estimation schemes (beyond Count-Min) might yield gains.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
