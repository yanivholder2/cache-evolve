# Variant v002 Insight

## Helped

_(none — no dimension improved)_

## Hurt

- **hit-rate**: The hot-zone boundary is defined by Vec position rather than access recency, so every cold-zone promotion (remove + push to tail) shifts all hot-zone items one slot toward the head; items near the boundary silently re-enter the cold zone and must be hit again to earn protection, creating churn that pure LRU avoids by always moving any hit to the tail.
- **per-op-time**: The O(1) hot-zone bypass saves no real work because baseline ClassicLRU's hot items naturally live at the Vec tail — making their `remove(len-1)` already O(1) — while the added integer division and branch on every get introduce overhead the baseline never pays.

## Takeaway

The two-zone optimization is predicated on a cost model that does not match this implementation: baseline LRU's access pattern already keeps hot items at the tail where `remove` is O(1), so bypassing the move buys nothing on the fast path. Worse, freezing relative order within the hot zone lets cold-zone promotions gradually push boundary items back into the cold zone, slightly degrading hit rate compared to the always-promote semantics of pure LRU. Effective 2Q protection requires a stable zone marker (e.g., a boolean flag or a separate queue) rather than a position threshold that shifts with every insertion.
