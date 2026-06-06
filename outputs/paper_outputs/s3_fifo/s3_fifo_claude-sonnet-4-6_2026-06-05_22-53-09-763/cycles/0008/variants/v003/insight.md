# Variant v003 Insight

## Helped

- hit-rate: Moving main-queue items to the front on every hit proactively refreshes their recency position, preventing frequency-decremented items from silently drifting to the eviction tail between accesses — closing a gap that the second-chance `evict_main()` only patched reactively at eviction time.

## Hurt

## Takeaway

Unconditional move-to-front on main-queue hits yields a small but real hit-rate gain over the parent's in-place frequency increment. The improvement is narrow because `evict_main()` already recycles high-frequency items back to the front at the eviction boundary, meaning most hot items were already surviving; MTF only helps items whose frequency had been decremented low enough that the next hit arrived before they reached the tail. On a Zipfian workload where the hot set turns over slowly, those cases are uncommon, so the recency-refresh adds incremental rather than transformative value.
