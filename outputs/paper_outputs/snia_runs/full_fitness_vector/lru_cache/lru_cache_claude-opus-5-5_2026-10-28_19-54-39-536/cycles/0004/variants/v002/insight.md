# Variant v002 Insight

## Helped

- per-op-time (put): Put p50 and p99 both dropped, as the hypothesis predicted. `put` now makes a single `map.entry(key)` probe in place of the parent's `map.get` + `map.insert`. On the eviction path, `mem::replace` swaps the victim's key out of the node instead of cloning it before `map.remove`. A fresh insert while filling now costs one hash instead of two, and an evicting insert costs two instead of three. This confirms that the put path is priced by its hash operations.

## Hurt

- per-op-time (get): Get p99 rose sharply and get p50 drifted up slightly. `get` and `on_hit` are byte-for-byte unchanged from the parent, so no instruction on the get path explains this. The rise is either benchmark noise or an indirect effect of the new insert-then-remove order on the hash table's internal state. Under that order the new key can never reuse the victim's freshly freed slot, which may change how deleted-slot markers build up and lengthen get-miss probe sequences. Pre-sizing to `capacity + 1` leaves the bucket count the same at the traced power-of-two sizes, so table size is not the cause. Note also that the no-resize test checks `map.capacity()`, which does not detect an in-place rehash.

## Takeaway

Merging the existence check and the insert into one `Entry` lookup, and moving the evicted key out instead of cloning it, is a clean, policy-neutral win on put. Hit rate and node size are exactly unchanged, which confirms eviction order was preserved. The summary metric tracks get p50, though, so this change barely registers there. The get-p99 rise needs a re-run, or a check of get-p99 at a lower load factor, to tell noise apart from a hash-table side effect of the insert-before-remove order.
