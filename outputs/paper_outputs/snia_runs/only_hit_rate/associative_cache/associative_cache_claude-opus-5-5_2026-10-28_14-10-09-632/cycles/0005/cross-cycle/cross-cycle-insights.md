## Recurring wins

- Cold insertion is the most reliable lever. Placing new items below reused ones won in cycle 0001 (LRU/LIP). It won again with stronger results in cycle 0004 (GCLOCK + probation FIFO), where it helped both traces with no trade-off.
- More probation depth in small buckets keeps paying off. It partly recovered small-bucket hits in cycles 0002, 0003 and 0005, and in the GCLOCK lineage (cycle 0005) it did so with no regression on either trace.
- Coarse or frequency-aware recency (CLOCK, then GCLOCK) never cost hit-rate compared to exact LRU. Its advantage grows with bucket size, and it is the only lineage that has moved the large-bucket cells.
- A hit during probation is the only promotion signal that has held up. Designs that admit to main only on that signal filter web_0 scans and still protect hm_0 reuse.

## Recurring failures

- Weakening admission or promotion causes churn. Examples: bounded promotion with MRU insert (0001), warm/ghost admission (0003, 0005), non-zero counters on promotion (0005), and graded promotion through one slot (0004). Each let in or protected the wrong blocks, usually hurting web_0.
- Ghost admission has failed in two lineages and at two sizes. With enough coverage it re-admits web_0's single-return blocks. That makes it a trace split, not a tuning problem. It is retired.
- Self-feedback PSEL in the LRU/BIP line failed twice. First its memory was shorter than the reuse distance (0003). Then a symmetric update voted on the sample majority, not on whether sampling paid off (0004). Both times it drifted and traded hm_0 against web_0.
- In LRU, stacking a probation floor on BIP-style MRU sampling is zero-sum (0003). Both split the same bucket space, so whatever helps one trace hurts the other.
- Fixed-fraction probation (`len/4`) is too short in 128-slot buckets for web_0's late second touches. This held in every lineage until a depth floor was added.
- Per-op-time and metadata-size have gone unmeasured in all five cycles. Every latency or cost claim is still untested, because the metric is missing from the fitness vector itself (a harness gap).

## Open questions

- How high can the small-bucket probation floor go before hm_0 starts losing main capacity?
- Does a larger probation fraction in 1024-slot buckets help web_0 at the large size, or does it weaken the filtering gained in cycle 0004?
- Can set dueling on `small_len` (leader buckets at different depths) pick the right depth per workload without a fixed rule?
- What explains the rest of web_0's small-size gap, given that depth only partly closed it and admission strength made it worse?
- What do GCLOCK sweeps, mid-vector inserts and miss-path ring scans actually cost once the harness reports per-op-time?

## Per-policy notes

- **LRU + LIP/BIP (cycles 0001–0004):** It won early but is now stuck. Every change after 0002 traded hm_0 (wants a large protected region) against web_0 (wants deep probation). The only item left worth trying is an asymmetric-PSEL fix; otherwise this line can be retired.
- **CLOCK → GCLOCK (cycles 0001–0003):** Hit-rate was even or better than its parent every time, and the gains were largest on hm_0 in large buckets. Ghost warm admission added nothing here.
- **GCLOCK main + probation FIFO (cycles 0004–0005, current lineage):** This is the first design to help both traces. Main is frequency-ranked and its sweep runs only on promotion, so it needs fewer slots than an LRU protected segment, and the probation/main trade-off is not zero-sum. Keep the counter reset to 0 on promotion, since the position behind the hand already gives a full rotation of grace. The cycle 0005 parent adds an absolute depth floor, which moved small buckets only. Large-bucket cells have not changed since cycle 0004.
