# Variant v001 Insight

## Helped

- hit-rate (16384 configurations, web_0 and hm_0): Both small-cache configurations moved up a little, which is the direction the hypothesis predicted. When the stash bucket is full, the second hit is now promoted instead of deferred. The deferred item therefore no longer takes the last stash slot and pushes out a candidate still waiting for its second hit. The size of the recovery is far smaller than the hypothesis expected.
- per-op-time (`get` p50/p99): `get` improved slightly at both the median and the tail. Fewer items are deferred, so fewer of them reach the forced third-hit promotion. That promotion runs the full `remove_at` → `promote_to_target` → `demote_from_normal_bucket` chain inside a `get`. Hot items also spend one less access on the stash path. Reading the stash length next to the two normal-bucket lengths added no measurable cost to the hit path.

## Hurt

- per-op-time (`put` p99, and to a lesser degree `put` p50): The parent's main gain is lost. `put` p99 rose back toward the pre-deferral regression that the parent insight describes. In a cache running at steady state, the stash bucket is usually already full. So the new "stash is full → promote immediately" branch is taken for most both-full second hits. That brings back the demotion cascade into stash, the extra stash evictions, and the full-bucket cascades on later `put`s that the single deferral had been preventing.
- hit-rate (131072 configurations, web_0 and hm_0): Both large-cache configurations slipped a little and gave back the parent's large-cache gain. Immediate promotion again lets a twice-hit item push a target resident that had already proven itself out of the target bucket. Large caches had benefited from keeping those residents in place.

## Takeaway

A full stash bucket is not a useful sign of pressure for this gate. Under load the stash is nearly always full, so the condition rarely allows a deferral, and this variant behaves much like cycle 0004 v002 again. It gave up the parent's `put` p99 gain in exchange for a small shift in hit rate from large caches to small caches, and the summary stayed flat. For a stash-pressure gate to keep the deferral's latency benefit, it needs a signal that separates harmful deferrals from harmless ones. One example is whether the stash slot that would be evicted holds an item still waiting for its second hit, rather than one with no hits. The raw fill level does not make that distinction.

Note: the overlay source could not be listed or read in this session, so this analysis relies on the hypothesis's change description and on the fitness vectors of this variant and its parent.
