# Variant v003 Insight

## Helped

- (none): the reactive `hir_floor` did not lift hit-rate; the one trace where a larger second-touch window would pay off was already served by the parent's narrow Q.

## Hurt

- hit-rate: `enforce_hir_floor` demotes the LRU-LIR block — the coldest by inter-reference recency but still a genuine low-IRR resident — into Q whenever an admission/promotion drains the resident-HIR count below `capacity/20`, so those demoted blocks land at the eviction front and are lost; the cost of evicting real LIR blocks outweighed the wider fresh-second-touch detection window the floor bought.

## Takeaway

A reactive starvation floor is not a free version of SLRU's drain guarantee: enforcing a resident-HIR minimum by demoting the bottom LIR block still sacrifices a measured low-IRR block, and on workload-a that sacrifice cost more hits than the enlarged detection queue recovered, leaving v003 below even its parent. Every floor-shaped intervention tried — static window (cycle 0006/v001) and now lazy reactive minimum — regresses, indicating untuned LIRS cannot reach the incumbent by protecting the drain queue, so the LIRS-floor axis can be closed.
