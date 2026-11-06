# Variant v003 Insight

## Helped

- per-op-time (`get` p99): The `get` tail got a little better than the parent's. When the stash LRU is marked, the victim gate forces the both-full promotion on the second stash hit instead of the third. Those items therefore spend one less access on the stash path. They are found in target sooner, and the extra lookup the parent's deferral caused no longer lands on a later `get`.
- hit-rate (web_0 at 16384): This is the only configuration that moved in the direction the hypothesis predicted, and the move is very small. On the web_0 small cache, a deferral that would push out a marked stash LRU now promotes instead, so that marked item stays in stash a little more often.

## Hurt

- per-op-time (`put` p99): Part of the parent's `put` p99 gain was lost, though `put` p99 stayed well below cycle 0006 v001's level. Each time the gate refuses a deferral, the promotion runs `promote_to_target` and its MRU-demotion cascade. That pushes a proven target resident into stash, and the next `put` to that stash bucket has to evict. The rise means the stash LRU is marked often enough that the gate refuses a meaningful share of deferrals.
- per-op-time (`get` p50): The `get` median got slightly worse. Every second stash hit with target and probing both full now does extra work before branching: it checks whether the stash bucket is full, finds its eviction candidate, and reads that item's `lfu_counter`.
- hit-rate (131072 configurations, web_0 and hm_0): Both large-cache configurations gave back a sliver of the parent's gain. The rationale assumed that on large caches the stash LRU would mostly be a cold one-shot insert, so the gate would rarely fire. In practice it fires there too, and each forced promotion evicts a target resident that the parent's deferral would have kept.

## Takeaway

Gating the single deferral on whether the stash LRU is marked did not recover the small-cache hit-rate loss. hm_0 at 16384 did not move, and web_0 at 16384 barely moved. The gate did give back some of the `put` p99 gain and some of the large-cache hit rate. So protecting a marked stash LRU is not where the parent's small-cache loss comes from. What this gate really does is decide how often the both-full cascade runs, and that shifts tail cost between `get` and `put` without improving hit rate. The `lfu_counter` mark on the stash LRU is a poor signal for whether a deferral will cost a future hit.
