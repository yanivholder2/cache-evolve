# Variant v001 Insight

## Helped

_(none)_

## Hurt

- **hit-rate**: Changing the warm-all fallback from `len - 1` to `len - scan_depth` evicts the item at the *inner edge* of the cold window — the one closest to the warm zone and therefore most recently demoted — rather than the true LRU tail at `len - 1`; on a Zipfian workload this displaces items that still have residual reuse potential while sparing the absolute coldest item, which is a worse eviction choice than classic LRU.

## Takeaway

The hypothesis conflated "inner-edge-first scan order among cold candidates" with "inner edge = oldest." In this cache's layout higher index means less recently used, so `len - 1` is the genuine LRU victim in the warm-all fallback case — not `len - scan_depth`. Replacing `len - 1` with `len - scan_depth` is actually anti-LRU: it evicts the *most recently promoted* item within the scan window instead of the most stale one, explaining the regression in hit-rate. The LRU-among-warm principle is sound, but its correct index is `len - 1`, which the parent already had.
