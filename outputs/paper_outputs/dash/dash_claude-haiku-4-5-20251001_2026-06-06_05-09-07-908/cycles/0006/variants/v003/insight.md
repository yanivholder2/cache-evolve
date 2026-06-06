# Variant v003 Insight

## Helped

- hit-rate: Adaptive threshold with divisor 3 lowers the LFU counter requirement (threshold=8 on 25/75 baseline), permitting more candidates to promote from target to probing, improving placement of hot items in the upper tier.
- per-op-time: The relaxed promotion gate reduces contention and tier traversal depth by allowing items to reach their optimal tier faster, yielding substantial latency cuts across both get and put operations.

## Hurt

(none)

## Takeaway

Divisor 3 proves to be the right balance for the adaptive threshold formula on balanced-geometry workloads. By being less selective than previous attempts (divisor 4), it permits more items to earn promotion based on frequency, directly improving hit-rate while paradoxically reducing operation latency. This suggests the previous formula over-filtered promotion candidates, leaving hot items trapped in lower tiers longer than workload behavior warranted. The formula's adaptation to stash bucket count makes this improvement portable across cache geometries.
