# Cross-Cycle Insights

## Recurring wins

- **Sketch width improves admission precision**: Doubling sketch width (4→8x) directly reduced hash collisions and improved hit rate. Measurement fidelity matters more than logic changes.
- **Monotonic frequency accumulation preserves signal**: Early high-frequency items remain predictive. Attempts to refresh via periodic decay degrade performance by aggressively removing earned signal.

## Recurring failures

- **Gate threshold relaxation admits marginal candidates**: Relaxing strict `>` to `>=` allows tied frequency estimates to evict hot residents. When sketch precision is uncertain, selectivity protects residents better than permissiveness.
- **Frequency decay removes signal too aggressively**: Periodic halving of counters discards early-accumulated frequencies that are genuinely predictive, harming overall hit rate.

## Open questions

- Does hit rate continue improving with wider sketches, or is there a diminishing return above 8x?
- Are other sketch parameters (depth, counter saturation) worth tuning alongside width?
- Is the baseline's counter saturation at u32::MAX a limiting factor?

## Per-policy notes

- **Admission gate**: Baseline logic is sound—the gate's strictness is a feature, not a bug. Precision beats permissiveness. Improvements come from better frequency estimation (wider sketch), not from relaxing the logic itself.
