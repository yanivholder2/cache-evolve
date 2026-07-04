# Variant v003 Insight

## Helped

- hit-rate: No movement — and that is the point. Folding the boundary-eviction `put` path's `evict_item` (`remove(idx)`) + `insert(0, item)` into a prefix `rotate_right(1)` plus an in-place `mem::replace` reproduces the exact post-state (new probationary item at index 0, former prefix shifted up, same victim removed, protected suffix untouched, `protected_count` unchanged), so eviction order and the segmented working set are byte-identical.

## Hurt

- hit-rate: No regression. The fast path fires only when `protected_count < len`, and it drops the same oldest-probationary victim the original remove+insert dropped; the all-protected case (`protected_count == len`, victim at index 0) still uses the unchanged remove+insert fallback, so no admission or eviction decision changed.

## Takeaway

This is a behaviour-preserving data-movement optimization: the steady-state full-bucket admission collapses two O(bucket_size) memmoves and a `Vec` length round-trip into one prefix rotate plus an in-place replace, with no length change. Identical hit-rate confirms the rewrite is byte-equivalent, but its intended payoff lives on a `put` per-op-time axis this hit-rate-only fitness vector cannot measure — so the change is provably free here, not provably beneficial.
