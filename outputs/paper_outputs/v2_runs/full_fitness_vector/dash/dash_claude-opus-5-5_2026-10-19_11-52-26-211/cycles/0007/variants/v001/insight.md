# Variant v001 Insight

## Helped

- hit-rate: Hit rate rose to exactly the value the hypothesis predicted for cycle-6 v001's rescue. A stash victim from `stash_insert_with_rescue_hashed` is now placed in a free slot of its target or probing bucket instead of being dropped. All three stash insert sites go through the wrapper: the fresh `put` and both inserts in `demote_from_normal_bucket`. Matching the prediction exactly is the bookkeeping check the hypothesis asked for. The ported rescue behaves like the original, and keeping the `borrowed` counter exact does not change placement. Rescue writes only into normal buckets, so it never changes `borrowed`.
- per-op-time (get p50, relative to cycle-6 v001 only): get p50 landed below the rescue-only baseline named in the hypothesis. The `borrowed[s1] != 0` gate in `promotion_get` still skips some `s1` scans when the cache also uses rescue. The margin is small, though, so the hypothesis's "clearly below" criterion is met only weakly.

## Hurt

- per-op-time (get p50 vs parent): get got slower than v002 on the median. This change does not touch `promotion_get`, so the cost comes from the cache state that rescue creates. Normal buckets hold more items, so target and probing scans cost more on misses. The likely mechanism, which the evaluation cannot isolate, is that this partly cancels the `s1` skip the parent gained.
- per-op-time (get p99): get's tail grew sharply. With rescue filling normal buckets, a stash hit's `promote_to_target` is more likely to evict. That eviction runs the whole `demote_from_normal_bucket` → stash insert → rescue chain inside the `get` call. The cascade branch also hashes the cascaded item again through `stash_insert_with_rescue`.
- per-op-time (put p50 and put p99): put got slower. Its tail rose above v002 instead of below it, which **refutes** the hypothesis's put-p99 criterion. Removing the redundant hashes did help: `put_stash_at_index_known` takes `s0` as an argument, and the evictee's single `key_hash` serves both its counter decrement and its rescue target. But rescue adds more work to the same slow path. On every stash overflow, `put` now checks whether two normal buckets are full and does a `put_at_normal_idx` insert. That costs more than the rehash it saved.

(metadata-size did not change: `Item` is still 40 bytes and `Dash` is the same size. Rescue and the hash-passing refactor add no fields.)

## Takeaway

One hash per key really does let the counter and rescue share work: hit rate matched exactly, and the evictee's hash serves both purposes. But on its own terms, it does not make rescue cheap. The cost of rescue is not hashing. It is the extra normal-bucket work on the stash-overflow path, plus the fuller normal buckets it leaves behind, which raise both get tails and put tails. To keep rescue's hit-rate gain, the next change has to cut that extra work, for example by rescuing only into target, or by limiting how often rescue fills normal buckets. Trimming more hashes will not be enough.
