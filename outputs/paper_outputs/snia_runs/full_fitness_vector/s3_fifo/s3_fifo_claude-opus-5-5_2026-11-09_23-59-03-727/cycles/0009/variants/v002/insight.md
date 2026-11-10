# Variant v002 Insight

## Helped

(No dimension improved.)

- hit-rate: All four cells match the parent exactly. This confirms the change preserves the policy: `entry()`, then `get() == s`, then `remove()` drops the same live ghost that the parent's `get`-then-`remove` dropped. It skips stale tuples, whether vacant or holding a newer seq, the same way. This counts as a correctness check passed, not a gain.

## Hurt

- per-op-time: `put` p99 went up, which is the opposite of what the hypothesis predicted, and `put` p50 also rose slightly. The likely cause is that std's `HashMap::entry()` is not a free replacement for `get()`. It takes the key by value, and when the key is missing it calls `reserve(1)` before returning `Vacant`. The ghost-expiry loop hits the `Vacant` case on every stale tuple, and lazy deletion makes stale tuples common: every ghost hit in `put` removes the key from the index but leaves its tuple in the deque. `ghost_index` sits at its live bound of `main_size`, and tombstones from constant remove/insert churn use up its growth budget, so that `reserve(1)` can occasionally set off an in-place rehash of the whole table on a `put`. That is exactly the kind of rare, expensive event that shows up in p99. The probe saved on each live victim was too small to make up for it. `get` timings moved by about 1 ns, which is noise, because `get()` is byte-identical to the parent.

(metadata-size did not move: no type, field, or container changed.)

## Takeaway

Merging the ghost-expiry `get`+`remove` pair into one `entry()` call did not lower the `put` tail; it raised it. Removing one probe per expired live ghost is not the lever the hypothesis expected, and std's `entry()` adds a hidden `reserve(1)` on the many stale-tuple misses this lazy-deletion ghost produces. Future probe-count work on this path should use non-reserving calls, for example keep the `get` check and call `remove` only on a match, or use `remove_entry` and reinsert on a seq mismatch. Better still, follow the hypothesis's fallback and look at the `evict_main` sweep and the `retain` compaction as the tail drivers (H03).
