# Variant v001 Insight

## Helped

- per-op-time: get and put latencies measured lower, but flipping a single comparison operator (`freq > 1` → `freq > 0`) does not change the resident set size or the small+main scan lengths that dominate `get`/`find_entry_mut`, so this drop is almost certainly benchmark variance rather than a real effect of the change.

## Hurt

- hit-rate: promoting every small-queue entry on its first reuse admits weakly-reused, one-touch items into main, where they consume capacity and evict more durable entries — the stricter `freq > 1` baseline filtered exactly these, so loosening it lowered hit-rate instead of raising it.

## Takeaway

On workload-a the "canonical" first-reuse promotion rule is worse than the baseline's two-reuse threshold: requiring a second touch before graduating to main is acting as useful noise-filtering, not as an over-strict discard. The hypothesis that `freq > 1` was throwing away genuinely reused items is contradicted — the extra reuse requirement is protecting main from short-lived items.
