# Variant v002 Insight

## Helped

- (none) No hit-rate cell improved. The target cell (`web_0` at 128K) went slightly down instead of up. In the parent, a hit on the LRU-end target item that was then evicted before its second hit was a rare loss. Promoting that item to probing right away did not recover much from it.

## Hurt

- hit-rate (16K, `hm_0`): The biggest loss. With `TARGET_ANY_HIT_PROMOTES`, every target hit now calls `promote_to_probing`, not only hits on the MRU item. In a small, contended cache the probing bucket is usually full, so each extra promotion evicts a probing resident. That resident cascades through `demote_from_normal_bucket` into its own target and can push that target's LRU item into the stash. Before, the same hit was just an in-place `get_from_position` refresh that moved nothing.
- hit-rate (128K, `hm_0`): Same churn at lower intensity. The hypothesis's rationale had flagged `hm_0` as at risk. Its reuse was already handled by the parent's MRU-gated promotion, so the extra moves only displace probing residents that had earned their place.
- hit-rate (16K, `web_0`): Same mechanism. Items reused only once are now pushed into probing after a single target hit. They land at probing's LRU end (`insert_pos = Front`), so they are the next probing victim anyway, and evicting them costs another cascade.
- hit-rate (128K, `web_0`): The predicted gain did not appear. An item promoted from the target's LRU end lands at probing's cold end. One cross-bucket move is not enough protection, and the probing resident it displaces costs about as much as the promotion saves.

## Takeaway

In the parent, the "hottest item only" condition on target→probing promotion did real work. It filters out one-hit items from crossing into the neighbouring bucket, where every crossing evicts someone and starts a cascade. Removing the filter turned an in-bucket reorder into an eviction-causing move and lost hit-rate on every trace and size, most on the small `hm_0` cache. Neither delaying (cycle 0004 v002) nor speeding up (this variant) the target→probing trigger beats the parent. That suggests the trigger is near its optimum, and the cost lies in what each promotion displaces, not in when it fires.
