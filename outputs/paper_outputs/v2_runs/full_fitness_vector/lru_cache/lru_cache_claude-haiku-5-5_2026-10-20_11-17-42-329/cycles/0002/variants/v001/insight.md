# Variant v001 Insight

## Helped

- metadata-size: Packing the referenced flag into the spare top bit of the node's index field, rather than adding a padded bool, shrinks `Node<u64,u64>` well below the parent's size, so the per-entry overhead drops noticeably.
- per-op-time: Hits no longer relink the node, so `get` no longer pays for list unlink and push; the put path is cheaper because the bounded single rescue replaces the parent's promote-on-every-hit pointer work, bringing put p50 down.

## Hurt

- hit-rate: Replacing immediate promotion with a lazily set referenced bit, and allowing only one rescue per miss, approximates recency less precisely, so some re-referenced keys are evicted that the parent would have kept.
- per-op-time: `get` p50 did not improve despite removing the relink on hits, so the relink was not the main cost on that path and the change gained little there.

## Takeaway

Deferring recency work to eviction time (a referenced bit plus one bounded rescue) trades some hit-rate for cheaper operations and smaller nodes, and the trade is mainly visible on the put path. Because `get` barely moved, the next gain has to come from elsewhere on the hit path, not from further removing relinks.
