# Variant v002 Insight

## Helped

- hit-rate: This is preserved, not improved. The hypothesis wanted the new index to leave admission unchanged bit for bit, and it did. The sketch, the LRU list and the `incoming_freq > victim_freq` duel are untouched. Feeding the sketch the same `key_hash` value means swapping the std map for the `slots` table changed only how residents are located, not which keys get admitted.

## Hurt

- per-op-time: Hashing each `get` once instead of twice did not move the headline p50, and the tails got clearly worse, with `put` hurt most. There are two likely causes:
  - **Pointer-chasing on lookup.** `find` stores only a `u32` per slot and has to load `nodes[idx].key` to compare on every probe step, including misses on non-matching neighbours. SwissTable keeps keys inline and filters with control-byte tags, so it rarely touches a key that doesn't match.
  - **Expensive eviction.** An admitting eviction now does a second probe to locate the victim's slot, then runs `remove_slot`'s backward shift. That shift calls `key_hash` again on every entry in the run after the hole, so the hashes that were "saved" come back on exactly the rare, expensive path that sets p99.
- metadata-size: No change. `Vec<u32>` plus `mask` takes the same inline space as the std map's handle, and `Node` stays at 24 bytes. The heap the index table saves compared with the std map's `(K, u32)` buckets isn't part of this metric.

## Takeaway

Avoiding the duplicate hash on `get` is not where TinyLFU's per-op cost lies. An index table that stores only node indices adds a dependent load into `nodes` on every probe and needs rehashes for backward-shift deletion, and together these cost more than the second Fx hash they replace. This disproves the parent's guess that std `HashMap` lookup and hashing were the remaining bottleneck. If a custom index is tried again, it should store a hash tag (or the home slot) next to each index, so probes can skip mismatches and deletion can shift entries without calling `key_hash`.
