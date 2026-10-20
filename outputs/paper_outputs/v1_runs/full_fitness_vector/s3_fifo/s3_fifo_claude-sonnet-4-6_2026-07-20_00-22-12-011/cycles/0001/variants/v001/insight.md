# Variant v001 Insight

## Helped

- hit-rate: A smaller small queue preserved more of the fixed capacity for the main queue, letting hot entries survive longer before eviction on this skewed workload.
- per-op-time: With fewer entries populating the small queue at steady state, the linear scan through `small` (chained before `main` in every `get`) is shorter, reducing average lookup latency when most hits land in the larger main queue.

## Hurt

(none)

## Takeaway

Reducing the small-queue fraction from 1/4 to 1/10 improved both hit rate and operation latency on a skewed workload, indicating the original 1/4 allocation over-invested in admission filtering at the cost of main-queue residency. The accompanying latency drop suggests that a leaner small queue is not just a capacity trade-off — it also reduces the per-lookup scan burden paid before reaching the resident hot set.
