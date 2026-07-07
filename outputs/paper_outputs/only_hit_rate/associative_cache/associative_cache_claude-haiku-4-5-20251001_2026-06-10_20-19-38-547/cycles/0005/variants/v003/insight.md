# Variant v003 Insight

## Helped

(None measured in this evaluation)

## Hurt

- **hit-rate**: Collision-aware selective stash insertion with a threshold of collision_count >= 2 discards evictions from low-contention buckets, preventing recovery of items that were still valuable when stash capacity was available.

## Takeaway

Collision-aware filtering of stash entries yields no improvement over unconditional capture, suggesting that the threshold for distinguishing "high-value" from "low-value" evictions based solely on collision frequency is misaligned with actual recovery patterns in workload-a. Items from lightly-contested buckets are apparently still worth preserving in the stash, indicating that collision count alone is not a reliable signal for whether an evicted item will be re-accessed. The filtering strategy appears to have been too conservative: v001's blanket stash capture already achieves the same hit-rate at 25% capacity, leaving no room for additional filtering to improve outcomes.
