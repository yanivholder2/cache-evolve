# Variant v001 Insight

## Helped

- hit-rate: When target and probing are both full, a twice-hit stash item now stays in stash instead of evicting the target's LRU resident. The hot newcomer and the existing resident both stay cached, and the newcomer stays at stash MRU, where the first stash search finds it. The gain shows mostly on web_0 at the small size, where normal buckets are full most often. hm_0 at the large size, the cell the hypothesis flagged as a risk, slipped slightly. So the change trades a little large-cache retention for small-cache retention instead of being a clean gain.

## Hurt

- per-op-time: Tail latency got worse on both `get` and `put`, and `put` p99 got much worse. That undoes most of what v002 recovered. Removing the `promote_to_target` → `demote_from_normal_bucket` → stash-insert chain also removed the only way a twice-hit item leaves stash once its normal buckets are full. Normal buckets almost never free a slot under steady load, so deferred items tend to stay in stash for good. That raises stash occupancy, which is the risk the hypothesis named. More `put`s then land in full stash buckets and evict there, and every `get` scans fuller stash buckets before reaching target and probing.

## Takeaway

The number of cascades is not what drives the tail; the stash fill level is. v002's overflow helped because it cut cascades and also kept items flowing out of stash. Deferral cuts cascades by stopping that outflow, so the stash-pressure cost comes back on `put`. A future fix on this path should keep a way for marked items to leave stash (for example, demote them or bound how long a promotion can stay deferred) rather than just skipping the promotion.
