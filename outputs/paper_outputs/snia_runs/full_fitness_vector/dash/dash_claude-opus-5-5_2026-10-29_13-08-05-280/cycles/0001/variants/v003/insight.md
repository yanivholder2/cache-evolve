# Variant v003 Insight

## Helped

- hit-rate: Needing two stash hits before promotion (`PROMOTE_THRESHOLD = 2`) keeps one-hit scan keys out of target buckets, and the admission check against a full target's victim `lfu_counter` keeps reused residents from being pushed out. Hit-rate went up on all four web_0/hm_0 trace and size cells. The largest gain is on hm_0 at the large cache size, where the normal tier has room to hold a frequently reused working set once scan keys stop churning it.
- per-op-time (get tail): Most one-hit stash lookups now end in an in-place `get_from_position` touch instead of `remove_at` + `promote_to_target` + a possible `demote_from_normal_bucket` cascade. Fewer multi-bucket moves cut get p99 a lot. Put timings also dropped slightly, which fits with the stash churning less.

## Hurt

- per-op-time (get median, the summary statistic): Every hit now writes a saturating counter increment. When a stash hit qualifies for promotion into a full target, it also reads the victim through `eviction_candidate_index()`. This fixed bookkeeping on the common path makes median get a little slower, even though there are fewer expensive promotions.

## Takeaway

Reusing the idle `Item::lfu_counter` field as a stash-reuse gate keeps `Item` the same size (metadata-size did not change). It also turns the stash into a working admission filter: scan keys stay in stash, and only keys with proven reuse take target slots. The cost is a small fixed charge on every get in exchange for a much shorter tail. Two questions are worth testing next: whether the median penalty comes from the target/probing increments or from the victim comparison, and whether a larger cache (where the gain was biggest) would benefit from a higher threshold.
