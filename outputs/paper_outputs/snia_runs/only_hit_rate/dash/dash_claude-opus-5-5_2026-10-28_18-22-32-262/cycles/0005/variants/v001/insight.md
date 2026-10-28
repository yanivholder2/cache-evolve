# Variant v001 Insight

## Helped

- hit-rate (128K, `hm_0`): This cell gained the most. In the parent, a key promoted from target into probing landed at the probing bucket's LRU end (`Front`). In 2-slot buckets, that made it the first victim of the next promotion from a neighbour. `PROBING_PROMOTION_POS = Back` now evicts the older, unrefreshed probing resident instead. `hm_0` repeatedly reuses a small hot set, so those keys now stay in probing rather than bouncing between probing, target and stash.
- hit-rate (16K, `hm_0`): A clear gain for the same reason. In a small cache, neighbouring buckets compete harder for probing slots, so the parent's `Front` placement lost the newly proven key more often. Placing it at MRU protects it at exactly the point of contention.
- hit-rate (16K, `web_0`): A barely visible gain. At this size, few `web_0` keys get hot enough to reach probing, so placement there rarely matters. It does show that the change does no harm in the cell that has been fragile in earlier cycles.

## Hurt

- (none) hit-rate (128K, `web_0`) did not change. This is the cell the cycle-0004 insight pointed to, and the hypothesis partly cited it. `web_0`'s large-cache reuse is decided by stash→target admission (the gate and its bypass), not by keys being lost from probing, so placement in probing has no effect on it.

## Takeaway

Using the MRU end for target→probing promotions is a third placement fix that pays off, after the stash-demotion and stash→target fixes. Each time, the rule is the same: an item that just proved itself should not land in the victim slot. Unlike the gate tweaks, this one mainly helped `hm_0`, whose hot set reaches probing. It did not help `web_0` at 128K, so that cell still needs a change on the admission side, not a placement change.
