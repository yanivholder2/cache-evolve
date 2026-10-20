# Variant v001 Insight

## Helped

- hit-rate: Splitting recency into probationary/protected segments and evicting probationary-LRU victims first shields twice-referenced blocks from the scan/loop flushing that the `lirs:` workload uses to defeat plain LRU.

## Hurt

- metadata-size: The added `protected` flag on `Node` plus the second head/tail pair, `prot_len`, and `prot_cap` on `LruCache` widen both the per-node and per-cache footprints.
- per-op-time: Every `get` now branches on segment membership and may promote (unlink, re-flag, push, and possibly demote the protected LRU), and `put` chooses victims across two segments, adding constant work to both hot paths.

## Takeaway

Segmented-LRU is the right lever for this LIRS trace: a single probationary gate converts one-hit-wonder churn into a real hit-rate gain. But it is not free — guarding reused items costs extra per-node state and a promote/demote step on the get path, so the win trades directly against metadata size and per-op latency.
