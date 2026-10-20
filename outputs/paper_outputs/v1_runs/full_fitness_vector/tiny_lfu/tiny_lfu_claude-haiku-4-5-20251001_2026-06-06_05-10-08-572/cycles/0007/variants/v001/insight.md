# Variant v001 Insight

## Hurt

- **hit-rate**: Eliminating all candidate scanning and reverting to unconditional LRU-head eviction removes even the minimal frequency signal from victim selection, causing hit-rate to drop 0.27% (0.7354 → 0.7334) as low-frequency items are evicted more often than frequency-aware selection would permit.
- **per-op-time**: Contrary to expectations that removing the scanning loop would improve latency, put p50 latency degrades by ~46% (116ns → 170ns), likely because the code path simplification doesn't offset the loss of frequency-aware filtering, which normally avoids admitting problematic low-frequency items that cause cascading evictions.

## Helped

(None)

## Takeaway

Pure LRU eviction without any frequency-aware victim scanning is strictly worse than even minimal frequency-aware selection (N=2). This variant conclusively demonstrates that the frequency-aware scanning loop is not wasteful overhead, but rather a load-bearing optimization that maintains both hit-rate and latency. Removing all frequency signals during victim selection increases misses and paradoxically increases put-operation median latency, confirming that the admission gate alone (incoming_freq > victim_freq) is insufficient without informed victim selection—the algorithm must actively identify and evict the coldest item among candidates, not just blindly take the LRU head.
