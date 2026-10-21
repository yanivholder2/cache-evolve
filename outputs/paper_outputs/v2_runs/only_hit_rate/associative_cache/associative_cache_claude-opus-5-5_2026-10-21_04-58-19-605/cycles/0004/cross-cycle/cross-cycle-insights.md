## Recurring wins

- On workload-a, filtering newcomers is the main lever (cycles 1–4). New keys go in with low protection, either at a deep insert position or with the CLOCK bit cleared, and earn protection only through a later hit. Every cycle's winner either strengthened or refined this filter.
- The filter works on both policy lines: shift-based LRU with deep insertion, and in-place CLOCK/GCLOCK. Exact recency order inside a bucket is not needed to get the gain.
- Make the filter strict, then add selective grace. Once the strict filter has peaked, extra grace helps only when it goes to keys with evidence of short reuse, and only when that grace is capped. The CLOCK ghost ring that admits returning keys with counter 1 won cycle 4.
- Removing the per-item wall-clock timestamp (logical stamps) did not change hit rate. It is safe to drop for hit rate, though the size benefit has not been measured.

## Recurring failures

- Uncapped rewards for returning keys. When a ghost hit sent the key to MRU (cycle 2), hit rate dropped because the reward weakened the newcomer filter. How big the reward is matters as much as the signal that triggers it.
- Frequency-based protection beyond the newcomer filter. Three cycles in a row it gave little or nothing: victim choice by frequency at the eviction end (cycle 3), SLRU-style two-step promotion (cycle 4), and GCLOCK counters, which only helped CLOCK relative to plain CLOCK (cycle 2). Once the filter removes keys that are hit only once, frequency has little left to separate. Caveat: the cycle 4 promotion test was weak because the demotion flag was never reset.
- Pushing insert depth all the way to pure LIP (index 0) lost hit rate. Depth returns shrank from 1/2 to 1/4 to 1/8, then turned negative. Newcomers need slightly more than one miss of grace.
- Evaluation gap (process failure, all 4 cycles): per-op-time and metadata-size were never recorded. Every latency or size claim is untested, including the miss-path costs of shifts, ghost scans and hand sweeps.

## Open questions

- Latency and memory: once per-op-time and metadata-size are measured, which comes out ahead overall, CLOCK/GCLOCK + ghost (no shifts, but a ghost scan on each miss and extra keys per bucket) or the 1/8-depth LRU line (shifts on each miss)?
- Generalisation: does the aggressive, about one-miss filter, and the ghost gain, hold on traces with longer reuse distances or at other cache sizes? Every result so far comes from workload-a at a single size.
- Ghost ring size: is the best size smaller than 4, larger, or scaled to the bucket?
- Does capped ghost admission carry over to the 1/8 LRU line, for example by inserting a ghost hit one step deeper than a normal newcomer?
- Does a shared global atomic clock for logical stamps hurt multi-threaded throughput?

## Per-policy notes

- LRU with insertion position (midpoint, then deeper insertion): won cycles 1–3. Insert depth 1/8 is the tested optimum, so depth tuning is done. Each miss shifts about half or more of the bucket, and that cost has not been measured.
- CLOCK / GCLOCK: improved its hit rate every cycle, and beat baseline LRU even with latency as its stated goal. Its gap to the LRU line came from where the hand rests, not from frequency. Leaving the hand on the newcomer brought it close to 1/8 LRU. Adding a capped ghost ring made it the current leader. It is the strongest candidate to become the main line once latency counts.
- Ghost / history-based admission: helps only when the reward is capped (an extra sweep pass) and fails with full MRU promotion. So far it has been tested only on the CLOCK line.
- SLRU / frequency promotion: about noise on top of the 1/8 filter. Low priority unless it is retried cheaply with correct flag resets.
- Item layout (logical u32 stamps): neutral for hit rate. It could be combined with any line, but its size win has not been measured.
