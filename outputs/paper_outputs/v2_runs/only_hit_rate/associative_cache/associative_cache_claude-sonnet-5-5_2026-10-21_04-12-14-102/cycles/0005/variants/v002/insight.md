# Variant v002 Insight

## Helped

- (none observed; hit-rate is the only measured dimension and it did not improve over the parent)

## Hurt

- hit-rate: Letting a recency-stamp comparison pick the victim from the protected segment when its LRU entry is older than the probation LRU entry slightly lowered hit-rate versus always evicting probation first. Protected entries have already proven reuse, so evicting one on age alone seems to cost more than it saves. Probation-first eviction was already doing the useful job of discarding one-time arrivals.

## Takeaway

Protected-vs-probation eviction by raw recency did not beat the fixed probation-first rule on this trace, and the small regression suggests the segment boundary encodes reuse information that a single last-access stamp does not capture. Eviction-side tweaks on top of v001's 4-slot probation look close to exhausted. Note: I inferred the mechanism from the hypothesis text and the fitness vectors and did not inspect the overlay diff.
