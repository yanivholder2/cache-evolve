# Variant v003 Insight

## Helped

- hit-rate: Splitting the resident set into a protected segment (80 %) and a probationary segment (20 %) gives frequently-accessed items a stable sanctuary — once an item is promoted to protected on its second access, it can only be displaced by other hot items, not by a fresh wave of one-hit-wonders entering probationary.

## Hurt

## Takeaway

The 2-segment SLRU does move hit rate in the right direction on this trace, but the improvement is narrow. The baseline TinyLFU already blunts one-hit-wonder pressure through frequency-gated admission — a new item must out-score the LRU victim in the sketch before it can evict it — so SLRU's protection is partially redundant. The change is most valuable for access patterns where hot items are at risk of being bumped even *after* surviving the frequency filter; where the admission gate is already effective, the extra segmentation buys relatively little.
