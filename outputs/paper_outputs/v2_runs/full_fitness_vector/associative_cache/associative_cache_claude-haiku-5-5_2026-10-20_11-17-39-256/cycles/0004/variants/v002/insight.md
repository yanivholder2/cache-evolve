# Variant v002 Insight

## Helped

- per-op-time: The put-side victim search now scans an 8-slot rotating window instead of the full 32-slot bucket, so the full-bucket minimum scan that v003 identified as its put cost is shortened, and put p50 and p99 both moved down from the parent.

## Hurt

- hit-rate: Choosing the minimum-counter victim only within a window that rotates through the bucket means the evicted slot is sometimes not the bucket's global lowest-frequency entry, so frequency protection on workload-a weakens and the drop exceeds the tolerance the hypothesis claimed.

## Takeaway

Narrowing the frequency victim search to a rotating window buys put latency at a measurable cost in hit-rate, which suggests the parent's gain depends on seeing the whole bucket on each eviction. Any further put-side savings should come from cheaper bookkeeping for the global minimum rather than from shrinking the scan width.
