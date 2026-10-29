# Variant v003 Insight

## Helped

- hit-rate (128K, `web_0`): This cell gained the most. When a key that was evicted from the stash is put again and its fingerprint is still in the ghost ring, it now enters the stash already credited with one hit. Its first stash hit then promotes it into a full target, which the free-slot bypass could not do. In a big cache, `web_0` keys come back soon after being evicted out of the stash, while the ring still holds their fingerprints. Under the parent, each of those keys had to earn two stash hits all over again.
- hit-rate (128K, `hm_0`): Same mechanism. This cell did not move under the parent's bypass, and it rose here. So `hm_0` does have reuse just beyond what the cache holds. The parent's in-item counter could not see it, because the counter is lost on eviction, and the ghost can.
- hit-rate (16K, `hm_0`): A clear gain at the small size. On `hm_0`, enough re-referenced keys come back before the small ring overwrites their fingerprints that the early promotion pays off. These promotions displace residents whose entry was not backed by any ghost evidence, and that does not cost a net loss here.
- hit-rate (16K, `web_0`): Essentially flat. This is the cell the hypothesis rationale singled out, yet it was the one that barely moved. The ring is sized to `GHOST_FACTOR × stash slots`, so at 16K it covers only a short span of past evictions. `web_0`'s returning keys at that size seem to come back after their fingerprints have already been overwritten. It did not get worse, which is consistent with a ghost false positive or stale credit costing at most one early promotion.

## Hurt

- (none) No cell went down. Placement is unchanged, and so is the counter reset on demotion from a normal bucket. The only change in behaviour is which first stash hits pass the gate, so no existing path lost hits.

## Takeaway

Keeping the reuse signal outside the item, in a FIFO ghost of keys evicted from the stash, is the first counter-style lever in this lineage that lifts every trace/size cell at once. That fits the rationale: the lost information was the history that the parent's per-item counter dropped on eviction. The gain is still small, and it is weakest on `web_0` at 16K, where the ring's span, tied to stash size, looks too short. The natural next step is to test the ring's reach, for example a larger `GHOST_FACTOR` or a ring sized to the whole segment instead of the stash. Changing the gate threshold is less likely to help.
