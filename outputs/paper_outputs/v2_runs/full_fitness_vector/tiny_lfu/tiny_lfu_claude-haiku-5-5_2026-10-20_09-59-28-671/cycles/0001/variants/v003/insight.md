# Variant v003 Insight

## Helped

- hit-rate: Splitting the resident list into probation and protected segments (with promotion on probation hit and eviction from probation first) gave a small gain on lirs workload-a, consistent with one-hit keys being displaced before re-referenced keys; the margin is small enough that it may not generalize beyond this trace.
- per-op-time: get and put latency dropped, which the added pointer bookkeeping in `touch()` and `unlink()` does not explain; treat this as likely measurement variance rather than an effect of the segmentation.

## Hurt

- metadata-size: `Node` gained a `protected: bool` (padded into the node) and `TinyLfuCache` gained four list anchors plus `prot_len`, so per-entry and fixed cache metadata both grew, as the hypothesis anticipated.

## Takeaway

Making the segmentation real costs a bool per node and five extra words per cache, and on this trace it buys only a marginal hit-rate gain. The gain comes from `get()`/`put()` victim selection preferring the probation head, so protected entries are only displaced when probation is empty, and a key reaches protected only after a second hit.
