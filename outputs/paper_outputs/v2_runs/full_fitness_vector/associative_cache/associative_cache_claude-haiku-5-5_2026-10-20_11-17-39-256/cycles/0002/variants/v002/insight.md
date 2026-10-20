# Variant v002 Insight

## Helped

- per-op-time: Switching the ghost ring to a fixed-size array with a rolling index removes allocation on eviction, which is the cost the hypothesis tried to avoid; both `get` and `put` p50 came in below the parent, so the `put` overhead that v003's Vec-based ring added appears to be gone. The `get` improvement is not explained by this diff and, read together with v003's unexplained `get` rise, suggests that earlier `get` movement was not caused by the ghost structure. Single-run timings, so treat the size of the gain as uncertain.

## Hurt

- hit-rate: Growing the bucket-local ghost ring from four to sixteen keys did not lift workload-a hit-rate; it fell slightly against the parent. The midpoint-reinsertion path is unchanged, so the longer window did not let returning keys get protected more often.

## Takeaway

Lengthening the ghost ring from four to sixteen slots did not recover hit-rate on workload-a, which weakens v003's explanation that the short ring was expiring returning keys; the reinsertion idea itself looks like the weak part. Following the hypothesis's own rule, ring length should be dropped as an axis rather than tuned further. Note that metadata-size stayed flat because the size probe still does not count the ghost array, so that dimension is not informative for this change.
