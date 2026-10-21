# Variant v003 Insight

## Helped

- hit-rate: Inserting new entries just above the probation boundary, instead of at MRU, means one-touch scan entries are evicted first and can't flush the reused working set. Skipping the move for hits already in the most-recent quarter also keeps hot entries stable.

## Hurt

- No dimension is shown to have regressed. The fitness vector reports only hit-rate, so metadata-size and per-op-time were not measured for this variant. The hypothesis adds no per-entry metadata, so metadata-size is probably unchanged. The positional checks and boundary insertion may add some work to put.

## Takeaway

Segmentation by bucket position alone, with no extra per-entry state or ghost entries, is enough to improve the hit rate over plain move-to-back LRU. Scan resistance seems to come mainly from the insertion point. The promotion-by-position rule is probably secondary. The missing metadata-size and per-op-time measurements mean the cost side of this trade is untested.
