# Variant v002 Insight

## Helped

- hit-rate: Shrinking the small queue from `capacity/4` to `capacity/10` handed the freed slots to the main resident queue, letting hot items survive more second-chance passes before eviction and raising the measured hit-rate over baseline.

## Hurt

## Takeaway

On this workload the 25/75 small/main split was starving the main queue; moving toward the paper's ~10/90 ratio is a net win because a 10%-capacity admission queue still filters one-hit-wonders into the ghost queue while the enlarged main queue retains hot keys longer. The gain is modest, suggesting the partition ratio is a real but bounded lever — worth pushing further but unlikely to be the dominant knob alone.
