# Variant v002 Insight

## Helped

- hit-rate: Back insertion for target-tier hottest promotions gives proven-hot items immediate MRU eligibility when advancing to probing, reducing re-promotion latency and eviction risk.

## Hurt

(none observed)

## Takeaway

Extending MRU back insertion from stash-hit promotions to target-tier hottest promotions improves hit rate. Items that have survived to target and proven themselves hot enough for tier advancement benefit from the same recency-based positioning that improved stash-to-target promotions in v001, creating consistent MRU prioritization across all tier-advancing paths.
