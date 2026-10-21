# Variant v001 Insight

## Helped

- hit-rate: Going from narrow buckets to 32 buckets of 64 ways makes each set closer to fully-associative LRU, which reduced hash-imbalance evictions, but the gain is marginal.
- per-op-time (put): Put p50 and p99 both dropped, even though the hypothesis predicted slower puts. The fewer, wider buckets may have changed eviction or victim-selection behavior in a way that made puts cheaper. The diff does not show why.

## Hurt

- per-op-time (get): Get p50 and p99 both rose because each lookup now linearly scans up to 64 entries in a bucket instead of 16. This matches the predicted cost of more associativity.

## Takeaway

Quadrupling associativity on a fixed 2048-item capacity bought almost no hit rate on workload-a, so conflict misses are not what limits this baseline. Capacity or the LRU policy itself is the likelier limit. The hit rate improved only slightly, while the longer per-bucket scans made gets slower. Further gains probably need a different eviction policy rather than more ways.
