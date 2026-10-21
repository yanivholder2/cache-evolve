# Variant v003 Insight

## Helped

- hit-rate: In `DashSegment::put`, 15 of every 16 fresh keys now go to the stash LRU end (`InsertPosition::Front`) with counter 0. The second-chance scan in `stash_put` starts at index 0, so the next insert into that bucket evicts this fresh key first. One-touch scan and loop keys therefore mostly evict each other instead of pushing out demoted ex-residents, which still enter at MRU with `PROMOTE_THRESHOLD - 1` pre-credit and need only one more hit to return to the normal tier. A second effect follows from the same code path: index 0 is now almost always a zero-counter fresh key, so the scan stops on its first slot and no longer decrements the counters of credited residents further along. Their protection lasts longer than it did under v002's MRU insertion of fresh keys.

## Hurt

## Takeaway

v002's insight concluded that tuning victim choice inside a stash bucket adds little. This variant shows that changing where fresh keys land does matter, because insert position decides which item the scan reaches first. Under BIP, each stash bucket effectively splits in two. The LRU slot becomes a one-slot probation area that fresh keys churn through. The other slots act as a victim cache for ex-residents, and the scan no longer ages them. The cost is that a fresh key now survives only if it is reused before the next insert into its stash bucket (or if it is the 1-in-16 MRU insert). Keys whose reuse distance is moderate are therefore at risk. Next steps could be tuning `BIP_PERIOD`, or protecting more than one probation slot per bucket. This was measured on a single LIRS trace at a single cache size.
