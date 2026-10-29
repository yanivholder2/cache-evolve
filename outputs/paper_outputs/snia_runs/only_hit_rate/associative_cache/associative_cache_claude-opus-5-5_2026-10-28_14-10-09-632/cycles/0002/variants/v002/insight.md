# Variant v002 Insight

## Helped

- hit-rate (`hm_0`, mainly at 16384): Sending every 32nd new item per bucket to the MRU tail gives a sample of blocks a full bucket lifetime instead of v001's ~`len/4` probation window. On `hm_0` with 128-slot buckets, a real share of reused blocks have reuse distances longer than the ~32-slot window, so the sampled blocks survive long enough to be hit again and promoted. This recovered part of the small-bucket loss that the v001 insight described. `hm_0` at 131072 moved only slightly, since 1024-slot buckets already had a ~256-slot window wide enough for most of its reuse.

## Hurt

- hit-rate (`web_0`, both sizes, more at 131072): On `web_0`, the items sent to the MRU tail are mostly one-touch blocks. Each one now spends a full bucket traversal in the protected region before eviction, where v001 would have drained it through probation, and it pushes reused blocks toward the LRU end. With 1024-slot buckets, each of these dead MRU entries stays longest and v001 already had little reuse cut off by probation, so there was little to gain and the pollution cost dominated.
- (`metadata-size` and `per-op-time` were not measured. The new `u32` counter adds per-bucket state, not per-item state. The MRU branch skips the mid-vector `insert` memmove on 1/32 of misses, so any cost effect should be negligible, but this run did not measure it.)

## Takeaway

BIP-style occasional MRU insertion helps only where the probation window is shorter than the workload's real reuse distance (`hm_0` at small buckets). Where the window was already wide enough, or where most cold traffic is scan-like (`web_0`), it is a net pollution cost. The small net gain hides a trace split. A fixed per-bucket period of 32 doesn't adapt to either trace or bucket size, so the natural next step is to choose between LIP and BIP adaptively (for example, DIP-style set dueling) or to tie the MRU-insert rate to bucket size. Tuning the period alone is unlikely to fix this.
