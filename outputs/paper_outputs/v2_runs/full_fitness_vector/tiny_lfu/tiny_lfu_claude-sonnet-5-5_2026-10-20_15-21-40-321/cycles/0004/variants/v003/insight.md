# Variant v003 Insight

## Helped

- hit-rate: Raising `SKETCH_WIDTH_FACTOR` from 4 to 16 gives each row four times as many counters. Fewer unrelated keys share a counter, so the `incoming_freq > victim_freq` admission test sees less inflated noise. The gain is small, which suggests collisions were already a minor error source at the old width.
- per-op-time: Both `get` and `put` p50 and p99 improved. The hypothesis expected a cost from the larger table, and the code change cannot explain a gain: it only changes the width constant and adds no work to `increment` or `estimate`. A wider table also gives no obvious reason for fewer cache misses. The improvement is most likely run-to-run measurement noise or an environment effect, so it should not be credited to this change.

## Hurt

- metadata-size: The `Node` size that the metric summarizes did not change. The reported sketch heap grew in proportion to the width factor, though. It is excluded from the summary but visible in the per-type breakdown. This is a hidden memory cost, and the summary metric does not show it.

## Takeaway

Making the count-min sketch 4x wider bought only a marginal hit-rate gain at the cost of 4x sketch memory. Sketch precision is close to saturated for this trace at 2048 entries, so it is a poor lever compared with changing the admission policy. The per-op-time gain contradicts what the change could cause, so confirm it with a repeat measurement before building on it.
