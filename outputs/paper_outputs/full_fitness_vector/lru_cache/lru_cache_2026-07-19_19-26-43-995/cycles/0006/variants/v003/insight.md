# Variant v003 Insight

## Helped

## Hurt

- **hit-rate**: The rescue clears SEEN_BIT via `push_back_prob`, resetting the saved entry's promotion progress to zero — it now needs two more accesses to graduate instead of one — so on this workload the items displaced to make room for rescued entries turn out to be more valuable than the entries saved.
- **per-op-time**: The extra `unlink` + `push_back_prob` pair inside `evict_one()` fires on every eviction where the probationary head carries SEEN_BIT, adding pointer-manipulation work to the put eviction path and raising put p99.

## Takeaway

Moving the second-chance rescue to the eviction path is O(1) in pointer operations, but `push_back_prob` unconditionally clears SEEN_BIT, which inadvertently raises the effective graduation barrier for rescued items from one additional hit to two. The hypothesis assumed the rescue would preserve the item's "about to graduate" status; instead it resets it, making the rescue counterproductive on workload-a where many entries are accessed exactly twice in a narrow window before being displaced.
