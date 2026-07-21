# Cross-Cycle Insights

## Recurring wins

- **MRU-end insertion** (Cycle 1) improved hit rate and per-op latency simultaneously and has held up as a stable anchor through all subsequent cycles; it changes tier pressure dynamics significantly and must be treated as a baseline-level change, not an optional add-on.
- **Struct metadata pruning** (timestamp removal, Cycle 1) produced a dual dividend — smaller per-item footprint plus eliminated syscalls — with no hit-rate cost; the pattern suggests the baseline carries additional dead-weight overhead worth continuing to audit.
- **Stacking orthogonal wins is additive** (confirmed Cycle 2): gains from independent subsystems (insertion position vs. item struct vs. promotion gating) compose cleanly with no interaction penalty.
- **Vacancy-gated promotion** (Cycle 3) is the only frequency-promotion variant that achieved a regression-free hit-rate lift; structural elimination of cascade opportunity outperforms threshold tuning as a way to control eviction side-effects.

## Recurring failures

- **Threshold-only calibration of the frequency gate** always hits a latency ceiling: without a vacancy guard, any promotion into a full tier triggers cascade evictions regardless of how rarely the gate fires. Raising the threshold softens but never cures the regression.
- **Unguarded stash-bypass on the MRU-end base** (Cycle 3) reversed a Cycle 2 win because MRU-end insertion makes target saturation the common case, so the bypass fires far more often and amplifies spillover into the probing tier. Interaction between insertion position and bypass-fire rate was hidden until the base changed.
- **Overlay coverage gaps** (Cycle 4): child overlays that modify only new files silently revert multi-file parent changes (especially struct reductions in `item.rs`), inflating per-item footprint and invalidating measurements. Two of three Cycle 4 variants failed partly for this reason.
- **Joint-condition mechanisms that fire too rarely** (stash-bypass gated on target-full AND probing-vacant, Cycle 4; lfu_counter reset for stash re-entrants, Cycle 4): on this workload and cache size, both preconditions are uncommon joint events; the hypotheses were structurally sound but the scenarios they target are not live patterns here.
- **LFU eviction override** (Cycle 1 v002): never received a clean fitness signal due to simulator timeout; has remained untested across all cycles.

## Open questions

- Can vacancy-gated promotion be combined with a threshold of ≥ 3 to filter for stronger reuse evidence while preserving the cascade-free guarantee — and does that push hit-rate above the ≥ 2 result?
- Is stash-bypass recoverable on the MRU-end base if gated on probing vacancy in addition to target fullness, mirroring the structural fix that made vacancy-gated frequency promotion succeed?
- Are there additional struct fields or per-operation syscalls beyond `timestamp` that carry no eviction advantage on this workload?
- Can a pre-flight check confirm that a child overlay inherits all files modified by its parent before the variant is evaluated, to prevent coverage-gap regressions from polluting future cycles?
- Now that promotion gating and struct overhead appear near their local optima, does meaningful headroom remain in **eviction ordering within a tier** (which item is displaced when a bucket must eject) or in **probing-tier promotion paths** (items accessed in probing rather than target)?
- Are the p99 latency regressions from frequency-gate variants workload-size-dependent — would they vanish at a different cache-to-working-set ratio?

## Per-policy notes

- **LRU (baseline insertion at LRU-end):** actively works against the tier hierarchy's warm-up logic on this workload; MRU-end insertion is a strict improvement and should be treated as a required base change going forward.
- **Frequency-gated promotion (LFU-ish gate, ≥ 2 threshold):** the workload contains a real cohort of reusable non-hottest items that the gate captures; ≥ 2 appears close to the threshold optimum on this trace (≥ 3 loses real items). However, the gate is only safe when vacancy-guarded — without it, cascade churn reliably appears.
- **Vacancy-gated policy (combined MRU-end + timestamp-free + vacancy gate at ≥ 2):** the most stable compound strategy across all cycles; no regressions observed when implemented correctly with full parent-file inheritance.
- **LFU eviction override:** merit remains unknown — the one attempt timed out and no subsequent cycle has retested it at a smaller scale or trace.
- **Counter-carrying / stash re-entry (lfu_counter reset):** oscillation between stash and target is not a detectable live pattern on workload-a at the current cache size; counter-state manipulation on stash promotion produced no measurable signal.
