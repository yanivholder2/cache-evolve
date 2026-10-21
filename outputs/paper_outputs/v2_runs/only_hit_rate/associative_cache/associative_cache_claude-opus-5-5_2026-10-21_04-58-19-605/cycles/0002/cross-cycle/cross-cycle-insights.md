## Recurring wins

- Low protection on insert is the strongest lever on workload-a, and it has held up in both cycles. New keys enter weakly protected (midpoint or deeper in the LRU order, or with the CLOCK bit cleared) and earn protection only after a second hit. This filters out one-off keys. Every hit-rate winner so far uses it.
- Making the filter stronger keeps helping. Moving insertion from the midpoint (cycle 1) to deeper toward the eviction end (cycle 2) raised hit rate again.
- Exact recency order inside a bucket is not required. CLOCK, and GCLOCK after it, beat the baseline hit rate without strict LRU order, so cheaper in-place hit paths are an option.
- Frequency signal adds value over a single recency bit. A small saturating counter (GCLOCK) improved on plain CLOCK.

## Recurring failures

- Anything that weakens the newcomer filter loses hit rate. Ghost-list admission that sends returning keys straight to MRU did worse than its parent. On this trace, one recurrence at a moderate reuse distance does not predict near-term reuse.
- Per-item wall-clock timestamps do nothing for hit rate on the eval path. Cycle 1 recommended dropping wall-clock ordering work.
- The evaluation gap has persisted for two cycles. Per-op-time and metadata-size have never been recorded. So every latency or size claim is untested, including CLOCK's cheaper hits, the smaller Item record and every added miss-path cost (Vec shifts, ghost hashing and scans, longer hand sweeps). This is the most important process failure to fix.

## Open questions

- Where is the best insert depth? Bracket between about 1/8 and pure LIP (index 0) to see whether hit rate peaks before index 0.
- Can the CLOCK/GCLOCK line match deep insertion's hit rate? Options are giving newcomers protection below that of a single hit, or raising the counter cap. If it can, it would win on hit rate and on (still unmeasured) latency.
- What is the true per-op-time cost of insertion that shifts elements on every miss, compared with CLOCK sweeps? This can only be answered once timing is collected.
- Would a shared global atomic logical clock hurt multi-threaded throughput?
- Could a much more conservative ghost policy recover any gain, such as partial promotion or a smaller ring holding only recent entries? This is low priority.

## Per-policy notes

- **LRU with insertion position (midpoint, then deeper):** best hit rate in both cycles. Its miss-path cost of shifting about half the bucket or more on every miss is still unmeasured.
- **CLOCK (bit cleared on insert):** beat the baseline hit rate even though it was written as a latency play. It trailed the insertion-position line.
- **GCLOCK (saturating counter):** improved on CLOCK but still did not reach deep insertion. It is the natural base for a hybrid with in-place hits and stronger newcomer filtering.
- **Ghost list with MRU promotion:** a hit-rate regression. Drop it unless it comes back in a much more cautious form.
- **Compact Item (logical u32 stamps):** left hit rate unchanged, as predicted. Its size benefit has not been measured, and it is worth stacking onto the winning lines.
