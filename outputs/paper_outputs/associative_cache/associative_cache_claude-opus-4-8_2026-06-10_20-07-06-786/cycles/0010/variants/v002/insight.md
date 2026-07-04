# Variant v002 Insight

## Helped

- hit-rate: No movement, by construction. The all-protected `put` branch's `evict_item` (`remove(0)`) + `insert(0, item)` is replaced by a single `mem::replace` at index 0; both memmoves over `[1, len)` exactly cancelled in the original, so the post-state is byte-identical — same protected victim at the LRU end (index 0) dropped, suffix untouched, and `protected_count` decremented by exactly one to mirror `evict_item`.

## Hurt

- hit-rate: No regression. The rewrite only touches the `protected_count == len` branch; the new item still lands probationary (`lfu_counter = 0`) at slot 0 and the same victim leaves, so no admission or eviction decision changed.

## Takeaway

Replacing the surviving slow branch with an in-place `mem::replace` makes the all-protected full-bucket admission O(1), completing v003's coverage of the entire full-bucket put path. The exact hit-rate tie confirms byte-equivalence, but the intended payoff is on a put per-op-time axis this hit-rate-only fitness vector cannot observe — so the change is provably free here, not provably beneficial.
