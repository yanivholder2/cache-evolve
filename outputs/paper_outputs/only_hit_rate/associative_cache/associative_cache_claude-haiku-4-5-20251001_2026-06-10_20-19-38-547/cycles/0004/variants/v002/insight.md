# Variant v002 Insight

## Helped

- **hit-rate**: LRU-ordered stash preserves recently-accessed items longer than insertion order would, keeping hot items in the secondary lookup layer where they can be re-hit after primary bucket eviction.

## Hurt

(none observed)

## Takeaway

LRU eviction in the stash successfully outperforms FIFO by aligning stash retention with item utility. When items are evicted from the primary bucket but later re-accessed, LRU keeps them available in the stash while FIFO would have discarded them. The 25% stash capacity with LRU tracking improves overall hit rate by capturing this secondary access pattern—items that matter enough to be re-requested are preserved longer than items only visited once.
