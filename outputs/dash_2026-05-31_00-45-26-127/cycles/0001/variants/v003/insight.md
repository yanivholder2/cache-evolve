# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Picking the lowest-`lfu_counter` victim inside the cold quartile spares items whose counter was bumped long ago and have since aged forward — on lirs:multi1 those "historically warm but currently cold" items don't get re-referenced, so protecting them over the position-0 victim costs a tiny amount of hit rate vs. baseline's blind front-pop.

## Takeaway

Reusing the per-item `lfu_counter` as a tiebreaker inside the recency-cold front is not free: the counter is a lifetime hit total, not a recency-bounded frequency, so as items drift from the back toward the front their stale counts argue against eviction even when their hit history is no longer predictive. On a recency-dominated mix like lirs:multi1, a cold-quartile min-counter scan picks the wrong survivor often enough to nudge hit rate down rather than up — a frequency signal in Dash buckets needs either decay or a windowed counter to beat plain LRU front-pop.
