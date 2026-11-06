# Variant v001 Insight

## Helped

- hit-rate: Putting new ClassicLRU items a quarter of the way up from the eviction end, rather than at the MRU end, means a one-hit scan item gets evicted after about a quarter of the bucket's turnover. The reused working set behind it survives. The gain is clear on both traces at the larger cache size (1024-item buckets). At the smaller size (128-item buckets) the gain is marginal on web_0 and modest on hm_0, because a quarter-depth slot gives a new item very little time to earn its first hit there.

## Hurt

- per-op-time: Put latency went up, mostly at the tail. `Vec::insert(len/4, ..)` shifts about three quarters of the bucket on every miss, where the old `push` shifted nothing. Every new-key put also pays a `name() == "ClassicLRU"` string comparison. Get latency is unchanged because the hit path (`get_from_position`) was not touched.

## Takeaway

Changing only where new items are inserted (LIP/BIP-style) gives scan resistance with no extra metadata (metadata-size did not move). The benefit grows with bucket size: large buckets act like a full LRU, which is where scans hurt it most. The cost is not free as the hypothesis assumed. A mid-bucket `insert` costs more than an append, so the put tail grows. Next steps worth trying are tuning `INSERT_DEPTH_DIVISOR`, scaling it with bucket size, or deciding the policy check once at construction instead of comparing strings on every put.
