# Variant v001 Insight

## Helped

- hit-rate: Landing stash→target and target→probing promotions, and all demotions, at the MRU end (`InsertPosition::Back`) means an item that was just hit is no longer the next ClassicLRU victim (index 0) of the bucket it enters. Reused items now outlive never-hit puts instead of being evicted before them. Hit-rate rose in all four (trace, size) cells, and the gain was larger on hm_0 than on web_0, so the baseline's Front landing was hurting the trace with more reuse the most.

## Hurt

- per-op-time: Contrary to the hypothesis's "`push` is cheaper than `insert(0, ..)`" prediction, get p50 and put p99 both got worse. A likely cause is that MRU landing puts a freshly promoted item exactly where `promotion_candidate_index` ("hottest") looks. Its very next hit then fires a target→probing move, which can start a probing→target→stash demotion cascade, where before it was a cheap in-place touch. More hits take the multi-bucket path, and more demoted items end up in stash, where `put` also has to work. That cost outweighs the cheaper single insert.

## Takeaway

In this promotion design, the insert position also sets how fast items climb tiers, not just eviction order. Back landing turns "promoted once" into "next to be promoted again", so the hit-rate gain comes with extra movement on every hit. A follow-up could keep `demote_pos = Back`, which is what protects reused items, and test `promote_pos` separately. It could also require a second hit before target→probing, to win back the per-op time without giving up the hit-rate gain.
