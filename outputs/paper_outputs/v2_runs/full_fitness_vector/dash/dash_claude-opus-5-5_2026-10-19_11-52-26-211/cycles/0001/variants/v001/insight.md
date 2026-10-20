# Variant v001 Insight

## Helped

- hit-rate: Items pushed out of target/probing now go into the stash at the MRU end (Back) instead of index 0 (the ClassicLRU victim). They now outlast never-reused `put` arrivals instead of being evicted by the very next stash overflow. On workload-a this rose above the hypothesis's 0.6634 bar, which suggests demoted items are often re-read.
- per-op-time: The stash-placement change could plausibly speed things up a little: appending at Back avoids shifting the bucket that a Front insert does, and fewer re-misses mean fewer stash→target promotion cascades. But the drop in both `get` and `put` latency is much larger than that change explains, so treat it mostly as variation between benchmark runs until it is re-measured.

## Hurt

(none — metadata-size did not change, since only positions within a bucket changed. No fields or types were added.)

## Takeaway

Where demoted items enter the stash matters on this workload. Treating the stash as SLRU's probationary segment, with protected-tier evictees at its MRU end, beats ranking them below cold new arrivals. Promotions still go to Front and the one-more-chance retry still uses `insert_pos`. So only the stash-entry position was tested here, and its effect on hit rate is isolated from the promotion-placement policy. That makes promotion placement (e.g. `get` passing Back) a separate lever still to test.
