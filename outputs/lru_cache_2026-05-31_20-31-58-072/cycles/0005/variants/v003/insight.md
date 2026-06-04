# Variant v003 Insight

## Helped

- (no fitness dimension moved in S4LRU's favor on the sampled workload)

## Hurt

- hit_rate: requiring three separate hits to climb L0→L1→L2→L3 means a key only reaches the protected top quarter after being re-referenced multiple times; on lirs:multi1 at size 512 most warm keys are evicted from L0's tail before they can earn that promotion, leaving the cache effectively operating with a ~capacity/4 retention window.
- hit_rate: every overflowing promotion is paired with a demotion back to the source level, so each rung's effective capacity is smaller than its nominal `capacity/4` whenever the workload exercises the promotion path — the conservation invariant that makes the structure clean also caps how much room recency-binning can buy.

## Takeaway

A flat four-level segmented LRU built on the baseline's arena pattern lands below the CLOCK-Pro-lite plateau the cycle-4 cycle-insight flagged: the rationale's expectation that S4LRU would "match or beat" it does not hold on lirs:multi1 at 512. The bottleneck is the multi-hop promotion ladder — only items hit at least three times survive into the top level, and the paired demotion on overflow keeps each level narrower than its declared cap. Recency-segmenting without a ghost ring or some asymmetric (e.g. larger top-level) split appears to leave too little protected room for this trace's re-reference distance.
