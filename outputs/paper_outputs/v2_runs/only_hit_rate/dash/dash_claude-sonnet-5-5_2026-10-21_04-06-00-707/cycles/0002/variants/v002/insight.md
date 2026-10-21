# Variant v002 Insight

## Helped

- None measurable. hit-rate is the only dimension reported, and it did not move relative to the parent. The hypothesis let stash hits be promoted into the probing bucket (with `InsertPosition::Back`) when it had spare capacity, but on workload-a that produced no observable gain over v001's MRU promotion.

## Hurt

- None measurable. Only hit-rate was evaluated, so metadata-size and per-op-time are unknown. The extra eligibility check in `promotion_get` could cost a little time. I could not read the overlay source or diff it against the parent, so this rests on the hypothesis text and the fitness vectors.

## Takeaway

Relaxing probing-bucket promotion eligibility added nothing on top of v001's MRU promotion. The gain the hypothesis called "marginal" in cycle 0001 looks like noise, or it is already covered by the MRU placement change. The v001 change is the lever that matters. Further tuning of promotion eligibility is unlikely to pay off on this trace.
