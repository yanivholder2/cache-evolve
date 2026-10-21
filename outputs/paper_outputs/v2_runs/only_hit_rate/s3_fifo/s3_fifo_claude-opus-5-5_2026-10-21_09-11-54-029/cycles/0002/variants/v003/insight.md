# Variant v003 Insight

## Helped

- hit-rate: Halving the small queue from 10% to 5% of capacity (204/1844 → 102/1946 at capacity 2048) gives 102 more slots to the main queue, which holds reused keys under the `freq > 1` second-chance rule. The ghost queue is sized to `main_size`, so it also grows and can send more demoted keys straight back to main. On workload-a, this still outweighs the cost of a probation window half as long, but the gain over v001 is small.

## Hurt

(No measured dimension regressed. The fitness vector reports only `hit-rate`. `metadata-size` and `per-op-time` were not measured.)

## Takeaway

Pushing the partition sweep past the paper's 10% still helps on workload-a, so hit-rate had not peaked at 10%. However, the gain is small, which suggests the curve is close to flat in this region. With only 102 small-queue slots, a key must be hit twice within about 102 insertions to graduate. Keys that miss this mostly survive because the ghost re-admits them to main on their next access. In this design the ghost queue, not the small queue, is doing most of the filtering. Further shrinking is unlikely to pay off much, and every extra ghost slot lengthens the linear `ghost.iter().any` scan on each `put` miss. `per-op-time` should therefore be measured before trading it for this small hit-rate gain. The result also comes from a single trace at a single cache size, so it may not generalize.
