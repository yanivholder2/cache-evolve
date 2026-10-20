# Variant v003 Insight

## Helped

- None measured. The only dimension reported for v003 is hit-rate, and it moved down from the baseline.

## Hurt

- hit-rate: The hypothesis replaces the fixed small-queue size with a `small_target` that moves one slot per ghost hit, clamped to 10–30% of capacity. On this trace, the adaptive target did not settle on a size that retains more keys than the fixed split. The likely cause is that the ghost-hit signal is noisy, so the target drifts instead of converging.

## Takeaway

Making the small-queue size adaptive, with a ±1 step per ghost hit and a 10–30% clamp, does not beat the fixed split on lirs workload-a at size 2048, so this sizing signal does not help hit-rate as implemented. The result is a single-trace, single-size measurement, and the metadata and per-op-time cost of the extra counter was not measured for this variant, so the change cannot yet be judged on overhead. The source diff could not be inspected during this run, so the attribution of the drop to the step rule is inferred from the hypothesis, not confirmed from the code.
