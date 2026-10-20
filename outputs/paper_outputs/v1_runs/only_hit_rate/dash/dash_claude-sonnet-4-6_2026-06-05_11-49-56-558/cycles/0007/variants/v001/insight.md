# Variant v001 Insight

## Helped

- hit-rate: Replacing ghost-set re-admission with unconditional second-chance recovery kept stash-overflow victims in the cache immediately rather than depending on a future re-insertion to reclaim a slot, plugging the stash loss site for items that would never be re-inserted by the workload.

## Hurt

## Takeaway

Swapping the ghost-set for immediate second-chance promotion at stash overflow produced a clear win over the parent's deferred re-admission scheme. The ghost-set only helps items that are re-inserted after eviction; second-chance keeps the item resident unconditionally at the moment of overflow, which is more robust on workloads where evicted items may not recur quickly enough to benefit from a ghost hint. The result confirms that the stash overflow site is a live loss source for this workload and that closing it eagerly outperforms the speculative ghost approach.
