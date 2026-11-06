# Variant v002 Insight

## Helped

- hit-rate: On the second stash hit, a full target with free probing space now takes the overflow path. The promoted item goes into probing, and the target's LRU resident is no longer pushed down to stash with its mark cleared. Every trace and size moved up, so the gain is consistent but very small. This means v003's MRU demotion had already rescued most of the items this change protects: a demoted target resident already got a full stash residency in which to earn its way back. Keeping it in the normal tier adds only a little on top of that.
- per-op-time: Tail latency improved on both `put` and `get`, and `put` p99 improved the most. This reverses most of the `put` p99 regression that v003 caused through stash pressure. The likely reason is that each overflowed promotion skips a whole `promote_to_target` → `demote_from_normal_bucket` → stash insert chain. That means fewer items enter stash, fewer later stash puts hit a full bucket, and fewer `get`s pay for the cascade. The predicted cost of a "third bucket" lookup did not appear. `promotion_get` already checks probing after target, so overflowed items are found by a branch that exists anyway.

## Hurt


## Takeaway

Combining probing overflow with MRU demotion technically meets H02's bar, but the hit-rate effect is marginal. Promotion placement and demotion placement are not independent levers: once demoted items land at stash MRU, saving them from demotion in the first place is worth little extra retention. What overflow mostly delivers here is fewer demotion cascades. That makes it a cheap way to recover the stash-pressure tail latency v003 introduced, more than a source of new hit rate.
