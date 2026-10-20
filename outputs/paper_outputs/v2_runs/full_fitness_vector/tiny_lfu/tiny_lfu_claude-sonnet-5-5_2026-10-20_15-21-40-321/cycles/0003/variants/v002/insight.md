# Variant v002 Insight

## Helped

- hit-rate: Choosing the colder of the two oldest entries as the admission victim lets a freshly-cold head survive when its neighbour is colder. The gain is marginal, though, and probably within noise for this trace.

## Hurt

- per-op-time: `put` on a full cache now does a second `mix` and `estimate` for the second-oldest node, plus a pointer chase into `nodes[head].next`. That raises put latency, especially in the tail.
- per-op-time: `get` latency also rose even though the `get` path is unchanged. That points to measurement noise or code-layout and inlining shifts rather than a real algorithmic cost, so the summary regression is partly spurious.

## Takeaway

Sampling two LRU candidates did not meaningfully improve admission quality on workload-a: the second-oldest entry is rarely colder than the head by enough to change the `incoming > victim` outcome. The extra per-eviction hashing therefore buys almost no hit-rate, and the per-op-time regression outweighs it. Further hit-rate gains probably need a different signal, such as sketch aging or a window or doorkeeper, rather than a wider victim sample.
