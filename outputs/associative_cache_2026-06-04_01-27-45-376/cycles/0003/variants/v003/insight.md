# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The `untouched >= 2` cardinality gate forces a fallback to slot-0 eviction whenever the bucket holds 0 or 1 untouched items, which sacrifices a touched (already re-referenced) LRU-front survivor instead of the lone untouched slot — and that lone untouched slot is more often a one-shot insert worth killing than a fresh keeper worth protecting.

## Takeaway

The "lone untouched slot is the freshly-inserted item at the back" assumption underspecified the regime: in a churn-heavy bucket the single `lfu_counter == 0` slot is just as often a stale one-shot the parent's filter correctly targeted, so promoting it to protected status costs more hits than it saves. Cardinality of untouched slots is not a reliable proxy for the all-touched regime in Dash buckets — the parent's unconditional untouched-first scan already degrades gracefully (fallback to slot 0) when no untouched slot exists, so adding a gate just removes correct evictions in the `untouched == 1` band.
