# Variant v002 Insight

## Helped

- hit-rate: The combination of SLRU segmentation and ghost-set fast-tracking narrowly improved hit rate — the protected segment shields recently-accessed items from eviction pressure that a flat LRU exposes, and the ghost ring-buffer allows re-admitted items to skip probationary staging and land directly in protected, reducing the penalty for items that were evicted and quickly re-requested.

## Hurt

_(no dimension moved negatively)_

## Takeaway

Introducing both an 80/20 SLRU split and a ghost ring-buffer over the single-segment baseline produced a real but small hit-rate gain on workload-a. The improvement is positive yet marginal, which points to a practical ceiling on the ghost-set mechanism at this buffer size: with the ghost capped at `capacity / 8` (256 slots at the 2048-item test size), rapidly evicted items can cycle out of the buffer before they are re-requested, limiting how often the fast-track path fires. Because SLRU segmentation and ghost-set admission were introduced together, the relative contribution of each is unresolved — a follow-on variant isolating the ghost set on top of a plain SLRU would clarify which mechanism drives the gain.
