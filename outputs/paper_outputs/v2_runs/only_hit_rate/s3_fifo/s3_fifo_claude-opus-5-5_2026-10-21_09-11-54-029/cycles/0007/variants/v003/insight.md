# Variant v003 Insight

## Helped

(No measured dimension improved. The eval measured only hit-rate, and it did not move from v002 at the reported precision. That is the result H03 predicted, not a gain.)

## Hurt

(No measured dimension regressed. Hit-rate stayed inside H03's ±0.0005 band. The change folds the 64-bit `DefaultHasher` output to `u32` (`h ^ (h >> 32)`, with 0 still mapped to 1). That only raises the chance that two keys share a fingerprint. In this code a shared fingerprint has one effect: in `put`, `ghost_map.remove(&fp)` succeeds for a key that is not really a ghost, and the key goes to the front of main instead of into small. Main holds at most 1946 live fingerprints, so a miss hits a false match with probability around 1946/2³². That is far too rare to show up on one trace. A second effect also leaves the policy unchanged: two keys that collide in `ghost_insert` overwrite one map entry, and `ghost_insert` already handles that case through its stale-slot check.)

## Takeaway

Halving ghost fingerprints to 32 bits is free on the policy side. Ring layout, hole-leaving removal and `ghost_head` advance are unchanged, and hit-rate is unchanged, so the "collisions are negligible" assumption holds at this cache size. H03's actual target was cutting ghost bytes from about 8 + 16 to about 4 + 8 per entry, which is `ghost_ring: Vec<u32>` plus `HashMap<u32, u32>`. That target is still unmeasured, so this variant can only be preferred over v002 once a metadata-size measurement exists. The per-miss SipHash fingerprint plus a second SipHash in `ghost_map` is still there, because H03 kept the default hasher on purpose. That leaves it as the next cost to attack on its own.
