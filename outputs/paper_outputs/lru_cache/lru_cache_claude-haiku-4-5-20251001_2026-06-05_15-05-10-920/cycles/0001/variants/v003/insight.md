# Variant v003 Insight

## Helped

(none)

## Hurt

- hit-rate: The middle-insertion strategy for re-accessed items places them at the midpoint of the LRU order rather than the tail, reducing their survival likelihood compared to standard MRU promotion and causing them to be evicted before re-access.

## Takeaway

The ghost-list "second chance" mechanism fails to improve hit rate on this workload. While the intuition—that recently-evicted keys deserve preferential treatment upon re-insertion—is reasonable, placing them at the middle of the linked list actually weakens their position relative to the hot tail (MRU) end. Re-inserted keys are less likely to be retained until re-access, suggesting either that the ghost list tracks items that don't actually re-appear frequently, or that the middle position is too conservative. The standard LRU policy of always inserting at the tail appears better suited to this access pattern.
