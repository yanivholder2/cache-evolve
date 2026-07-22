# Variant v003 Insight

## Helped

- **hit-rate**: Changing the cold-candidate scan in `eviction_candidate_index` from reverse order (`(len-scan_depth..len).rev()`) to forward order (`len-scan_depth..len`) causes the oldest never-accessed item in the window to be evicted first; that item has survived more eviction cycles and is less likely to be accessed soon, so fresher cold items near the absolute tail get a chance to warm up and produce hits before being evicted.

## Hurt

_(none)_

## Takeaway

Within the bounded second-chance scan window, scan direction matters: evicting the oldest cold candidate (inner-edge-first) outperforms evicting the newest cold candidate (tail-first) on this Zipfian workload. The LIFO-among-cold behaviour of the parent was quietly discarding items that had only just arrived and not yet had time to be re-requested, whereas the LRU-among-cold order in this variant preserves them long enough to become hits. The `metadata-size` and `per-op-time` dimensions are entirely unaffected because the change is a pure iteration-order flip with identical scan depth and fallback logic.
