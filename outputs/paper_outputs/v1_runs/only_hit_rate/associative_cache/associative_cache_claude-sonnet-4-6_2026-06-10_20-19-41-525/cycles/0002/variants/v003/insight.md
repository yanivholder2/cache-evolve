# Variant v003 Insight

## Helped

- hit-rate: Mid-point insertion prevented cold scan items from occupying the MRU back immediately on arrival, so warm recurrent entries were no longer pushed toward the eviction front by the scan-heavy segments of workload-a.

## Hurt

_(no measured dimension declined)_

## Takeaway

The LIRS workload-a trace contains enough scan-like traffic that protecting the MRU region from newly inserted cold items—by landing them at `items.len() / 2` instead of the back—produces a meaningful hit-rate gain without changing the eviction policy or bucket geometry at all. The improvement confirms the hypothesis: the baseline's vulnerability was not the ClassicLRU ordering discipline itself but the insertion point that gave every new item full MRU protection regardless of its actual reuse likelihood. One cost left unmeasured here is that `Vec::insert` at a midpoint is O(n) (it shifts the upper half of the bucket), whereas the baseline's `push` is O(1); whether that latency penalty matters at scale would need a `per-op-time` eval pass to confirm.
