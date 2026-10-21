# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Landing stash→target promotions at `InsertPosition::Back` makes the promoted item the `promotion_candidate_index()` right away, so it moves on to probing after only one more hit, where the baseline needed two (the first target hit only bumped it from Front to Back). Probing therefore admits items on weaker evidence. Each early target→probing move also evicts the probing bucket's LRU resident and sends it down the demotion cascade. That bucket is a neighbour's target, so more of the neighbour's proven items get pushed out.

## Takeaway

In this Dash design, where a stash-promoted item lands also sets how many hits it needs to reach probing, because the "is hottest" check reads its position. Putting it at MRU lowers that bar to one extra hit, and the extra cross-bucket moves cost more hit-rate than the protection inside the target bucket gains. A version of the SLRU fix that could work has to keep the probing admission bar where it was, for example by tightening the hottest check or needing a separate hit count, while still keeping new arrivals in the target bucket from being evicted first. (metadata-size and per-op-time are missing from this variant's fitness vector, so nothing can be said about them.)
