# Variant v002 Insight

## Helped

- hit-rate: Making a stash item get two hits in one stay before it moves into its target bucket keeps keys reused only once (scan and burst traffic) from evicting residents of the small 8-bucket normal tier. The gain is largest at the 16K cache size on both web_0 and hm_0, where the normal tier is most crowded. hm_0 also gains at 128K, so its reuse pattern still pays off from the filter at the larger size.

## Hurt

- hit-rate: On web_0 at 128K the result is flat to slightly down. With more room, first-hit promotion was already cheap there, so the extra probation hit mostly delays truly hot keys reaching the normal tier and leaves them in the stash longer, where stash LRU pressure can evict them.

## Takeaway

An SLRU/2Q-style two-hit gate on the stash-to-target move, built on the `lfu_counter` field that ClassicLRU never reads, is a net hit-rate win. It helps most when the normal tier is small compared with the working set and churn from single reuses dominates. When the cache is roomier the gain disappears, which suggests the gate should scale with capacity or occupancy rather than staying fixed at 2. This variant's eval measured only hit-rate. The claims that the change leaves metadata-size alone (no new field) and makes per-op-time cheaper (an in-place stash refresh instead of a cross-bucket move) have not been checked.
