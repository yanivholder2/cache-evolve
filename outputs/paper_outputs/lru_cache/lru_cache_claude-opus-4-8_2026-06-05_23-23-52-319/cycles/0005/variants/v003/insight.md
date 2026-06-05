# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Demoting protected keys to the probationary LRU (front, next-to-die) instead of
the MRU (back) left hit-rate exactly unchanged on workload-a: the eviction
victim is always `prob_head`, but where a demoted node lands relative to fresh
arrivals had no observable effect here. At the 95/5 split the protected segment
holds ~1945 slots and rarely overflows, so `enforce_protected_budget` fires
seldom; and the once-hot keys it does demote are cold enough that they earn no
second touch in the ~103-slot window regardless of whether they sit at its front
or back. This confirms the demotion-placement lever is inert on this trace — the
binding constraint is the probationary floor's *size*, not the order demoted
keys occupy within it.
