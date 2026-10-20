# Variant v002 Insight

## Helped

- hit-rate: Doubling `SKETCH_WIDTH_FACTOR` (8 → 16) cut collisions a little in the four conservative-update rows, so the strict `>` admission duel made slightly fewer wrong calls. The gain is tiny and falls inside the ≤0.0005 "lever closed" threshold the hypothesis set beforehand.
- metadata-size: The summary did not move, as predicted. The summary is `Node<u64,u64>`, and the change added no struct fields. The doubled table shows up only in the heap row reported for the sketch.

## Hurt

- per-op-time: `get` p50 went up even though the `get` code was not touched. Every `get` still increments the sketch, which means four random counter writes, and those writes now land in a table twice as big (≈128 KB at cap 1024, ≈256 KB at cap 2048). That table no longer fits comfortably in L1/L2, so more of those writes miss the cache. This is the cost the hypothesis asked us to check, and it is real.
- per-op-time: `put` p50 and p99 also rose, which fits the same pattern of extra cache misses on the sketch. The `get` p99 dropped at the same time. As the parent's insight found, tail numbers move both ways on small edits, so the p99 changes are not a trustworthy signal.

## Takeaway

On the CLOCK base, sketch width has stopped buying hit rate. The second doubling recovered almost none of the error left in the admission duel. It also made the sketch big enough that its per-access writes now cost latency, so the per-op-time summary paid a clear price for a hit-rate gain no bigger than noise. Treat the width lever as closed and keep the factor at 8. Any further hit-rate gain has to come from somewhere other than raw counter width, such as aging/reset, a doorkeeper, or the admission or eviction rule itself.
