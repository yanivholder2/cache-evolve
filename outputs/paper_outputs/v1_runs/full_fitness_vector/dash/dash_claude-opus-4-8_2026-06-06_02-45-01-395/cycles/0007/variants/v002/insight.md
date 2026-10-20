# Variant v002 Insight

## Helped

- per-op-time: Replacing the `HashMap` probe with `TargetHeat`'s one-index-plus-one-compare access dropped both tail latencies (get-p99 and put-p99) and put-p50 — eliminating the SwissTable group scan and `entry` bookkeeping removed the worst-case probe variance the parent still carried on every hot target hit.

## Hurt

- per-op-time: get-p50, the metric the hypothesis explicitly targeted, did not fall — it ticked up by a hair, so the direct-mapped array's median access is no cheaper than the parent's already-`U64Hasher`-folded HashMap probe.

## Takeaway

Collapsing the `target_heat` probe to a flat direct-mapped array refutes the hypothesis's premise that the SwissTable group scan was bounding get-p50: H01 had already removed the only median-path cost (the hasher), so what remained was tail variance, not median latency. The array's real win is tighter p99s, and it preserved hit-rate exactly at 0.6794 — confirming that 2×-capacity sizing makes slot collisions a negligible promotion-timing perturbation, so the structurally cheaper table is a safe, variance-reducing substitute even though it does not move the median.
