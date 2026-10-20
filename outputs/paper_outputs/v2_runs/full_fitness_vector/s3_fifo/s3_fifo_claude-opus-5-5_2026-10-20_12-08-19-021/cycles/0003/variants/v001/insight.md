# Variant v001 Insight

## Helped

- per-op-time: Both `get` and `put` latencies improved, but the change does not explain it, so the gain should not be credited to H01. `get` is byte-identical to the parent: one `index` lookup plus a slab access. `put` now does *more* work on every main eviction: an `mghost_push` with a hash insert and a deque push, plus a defensive `ghost_take`. The gain is most likely variance between benchmark runs or a change in the environment. Treat it as a reason to re-baseline v003 on the same host before comparing latencies.

## Hurt

- hit-rate: The ARC-correct signals scored a little below the parent. The likely cause is that both ghost paths now insert straight into `main`. Before, a key that `main` had evicted came back through `small` like any new key and had to prove itself there. Now `mghost` lets it skip that filter. The reversed small-ghost branch can also evict from `main` twice per hit: once to give up the slot, and once more to make room for the incoming key. Every one of those victims goes into `mghost`, which makes it likely to shrink `small` again later. So the two signals push against each other, and growing `small` doesn't pay off right away, because the extra slot stays empty until later misses fill it.
- metadata-size: The summary did not move, because it still counts only `Entry<K,V>`. But the fixed `S3Fifo` struct row grew because of the four `mghost*` fields. The second ghost also adds up to `capacity / 2` live `(K, u64)` records, up to twice that many physical records in the deque, and matching `mghost_set` entries. The dimension does not capture any of this cost.

## Takeaway

Moving v003's adaptation to the textbook ARC direction (each ghost grows its own side) and removing the graduation trigger did not beat v003's lopsided rule on workload-a. The hypothesis named the reversed small-ghost direction as the likely cause. But the new main-ghost path also lets evicted main keys skip the `small` filter, so this result cannot separate the two changes. The next step is to test them one at a time: keep `mghost` with only the main-grows signal, reinsert `mghost` hits into `small` instead of `main`, or restore v003's small-ghost direction. That will show whether the regression comes from the signal direction or from the admission bypass.
