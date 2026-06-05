# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Hard-coding `InsertPosition::Front` in `demote_from_normal_bucket` places recently-displaced items at the LRU end of their destination tier, where they are immediately vulnerable to eviction before a re-access can occur — items with residual warmth that previously survived by inheriting the MRU position (`Back`) from the triggering hot promotion are now expelled faster.

## Takeaway

Applying the classic segmented-LRU asymmetry (promotions at MRU, demotions at LRU) to this cache's demotion cascade hurt hit-rate rather than helping it. In the parent, demotions triggered by a hot `get` inherited `InsertPosition::Back`, meaning displaced items landed at MRU of their destination tier and gained additional residence time; this workload's reuse pattern is warm enough that those items were re-accessed before eviction. Pinning all demotions to `Front` removes that buffer, causing items with residual heat to churn out of the target and stash tiers faster than the workload's inter-access gaps allow. The textbook asymmetry assumes a clean hot/cold separation that workload-a does not have.
