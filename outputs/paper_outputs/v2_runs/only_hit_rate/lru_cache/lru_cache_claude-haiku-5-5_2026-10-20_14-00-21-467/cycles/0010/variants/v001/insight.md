# Variant v001 Insight

## Helped

- hit-rate: New keys are now linked just before a probation midpoint marker (the cold part, nearest the eviction end) instead of at the MRU tail, so one-hit entries age toward the victim window sooner and the hot part of probation is kept for re-referenced material.

## Hurt

## Takeaway

Insertion depth stacks on the segmented incumbent, but only marginally on workload-a: the gain is small, which suggests the protected/probation split already removes most of the one-hit pollution that midpoint insertion targets in plain LRU. The lever is worth keeping as a tuning axis, but it is not a large source of hit-rate on its own.
