# Variant v002 Insight

## Helped

(none — the change did not move any dimension upward)

## Hurt

- hit-rate: Capping the protected segment at `max_size/2` and demoting the oldest protected item back to the probationary front on every over-cap promotion strips proven (twice-accessed) keys of their protection far too aggressively, forcing them to re-earn promotion from scratch and exposing them to eviction again — the demotion churn discards working-set keys the parent kept protected, costing hit-rate.

## Takeaway

A tight protected cap with demotion-instead-of-eviction does not buy rising keys a longer probation lap for free — it pays for that lap by continuously knocking demonstrably-hot keys out of protection. On this workload the protected segment was load-bearing: shrinking it to half the bucket and recycling its LRU end back into probation regresses hit-rate below the parent, so the cap should be looser (or absent) rather than live.
