# Cross-Cycle Insights

## Recurring wins

- **Structural compaction of node metadata** (e.g., shrinking link fields to u32 sentinels) reliably reduces memory footprint and improves get latency through better cache-line utilization — without degrading eviction quality when the LRU chain is preserved.
- **Simpler structural changes outperform algorithmic complexity** when the baseline is already well-tuned: a lean node layout beat both a two-queue policy and CLOCK in overall latency.

## Recurring failures

- **Algorithm complexity that adds per-op bookkeeping** (extra pointer chasing, queue membership tracking, array sweeps) erodes structural gains and has not paid off net-positive in any cycle so far.
- **CLOCK-style eviction** introduced cache-line pollution via the sweeping hand, negating the structural win from flat slot arrays; coarser recency signals also hurt hit rate on this workload.

## Open questions

- Can the two-queue hit-rate improvement (v001) be recovered on top of the leaner v002 node layout, where per-queue bookkeeping fields represent a smaller fractional overhead?
- Is CLOCK's cache-line pollution inherent to the algorithm, or does a segmented or lazy-sweep variant recover the structural benefit without the latency penalty?
- How far can node size be reduced (toward a single cache line per node) before diminishing returns appear?
- Do put-path regressions observed in compaction variants indicate a systematic write-cost floor, or are they addressable independently of get-path changes?

## Per-policy notes

- **LRU (baseline + u32 sentinel links, v002):** Current best overall. Smaller nodes and preserved chain ordering jointly improved get latency; put latency regressed slightly from cast overhead but remained acceptable.
- **Two-queue (v001):** Hit rate improved by protecting hot items, confirming the workload has meaningful reuse structure. However, extra bookkeeping overhead offset structural gains — worth revisiting on a leaner node baseline.
- **CLOCK (v003):** Flat slot layout matched v002's structural compaction, but single-bit recency hurt hit rate and hand-sweep cache-line pollution blew up both get and put latency. Deprioritized until a cache-friendly sweep mechanism is proposed.
