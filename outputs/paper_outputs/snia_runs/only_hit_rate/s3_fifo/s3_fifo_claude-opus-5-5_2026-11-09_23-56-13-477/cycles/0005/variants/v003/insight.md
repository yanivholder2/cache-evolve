# Variant v003 Insight

## Helped

(none: no dimension in the fitness vector moved)

## Hurt

(none)

Neutral:

- hit-rate (all four trace × size cells): The result is bit-identical to the parent. This is expected, because every change is mechanical and leaves the policy alone:
  - **Hasher swap.** `FastHasher` replaces SipHash. The bucket hash only affects where keys sit in the table, not which key gets evicted.
  - **Narrower ghost stamp.** The stamp is now a `u32` and is only ever compared for equality. The ghost deque is compacted at `2 * main_size`, so a wrapped stamp can't match a live one.
  - **Reordered strong ghost-hit path.** It now runs `evict_main()` first, then `alloc(key)` without a clone, then updates the index in place with `get_mut`. This is safe because `evict_main` only removes resident keys, so the returning key's ghost index entry survives until it is re-pointed.

  The parent's weak-returnee ordering is kept: the index entry is removed *before* `evict_small()`. The unchanged `Reference` oracle still enforces equivalence.

## Takeaway

v003 shows that cycle-0004 v002's cost-only changes can be layered onto the freq-gated ghost policy without changing hit/miss behaviour. That makes it a safe base for later policy work in this lineage. There is one subtle point. Moving the strong ghost hit to "evict, then re-point in place" relies on main eviction never touching ghost entries, while the weak-returnee path still has to drop its ghost record before small eviction. The two branches need different orderings, and keeping both is what holds equivalence.

The fitness vector has only the hit-rate dimension, so this evaluation does not show whether the hypothesis's actual goal was met: lower per-op-time and metadata-size. The 8-byte `Loc` (down from 16, enforced at compile time) should shrink every index slot. The narrower ghost-deque element `(K, u32)` may save nothing for 8-byte keys like `i64`, because alignment pads it back to 16 bytes. The faster hasher and the dropped clone should reduce per-op cost. Those gains have to be confirmed by a cost-dimension evaluation before v003 can be credited with a win over the baseline.
