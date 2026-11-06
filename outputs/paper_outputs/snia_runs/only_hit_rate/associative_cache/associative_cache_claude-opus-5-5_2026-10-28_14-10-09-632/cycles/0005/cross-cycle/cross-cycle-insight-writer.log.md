# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-28T15:04:55+00:00

## timeline

### 15:05:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Cold insertion is the most reliable lever. Placing new items below reused ones won in cycle 0001 (LRU/LIP). It won again with stronger results in cycle 0004 (GCLOCK + probation FIFO), where it helped both traces with no trade-off.\n- More probation depth in small buckets keeps paying off. It partly recovered small-bucket hits in cycles 0002, 0003 and 0005, and in the GCLOCK lineage (cycle 0005) it did so with no regression on either trace.\n- Coarse or frequency-aware recency (CLOCK, then GCLOCK) never cost hit-rate compared to exact LRU. Its advantage grows with bucket size, and it is the only lineage that has moved the large-bucket cells.\n- A hit during probation is the only promotion signal that has held up. Designs that admit to main only on that signal filter web_0 scans and still protect hm_0 reuse.\n\n## Recurring failures\n\n- Weakening admission or promotion causes churn. Examples: bounded promotion with MRU insert (0001), warm/ghost admission (0003, 0005), non-zero counters on promotion (0005), and graded promotion through one slot (0004). Each let in or protected the wrong blocks, usually hurting web_0.\n- Ghost admission has failed in two lineages and at two sizes. With enough coverage it re-admits web_0's single-return blocks. That makes it a trace split, not a tuning problem. It is retired.\n- Self-feedback PSEL in the LRU/BIP line failed twice. First its memory was shorter than the reuse distance (0003). Then a symmetric update voted on the sample majority, not on whether sampling paid off (0004). Both times it drifted and traded hm_0 against web_0.\n- In LRU, stacking a probation floor on BIP-style MRU sampling is zero-sum (0003). Both split the same bucket space, so whatever helps one trace hurts the other.\n- Fixed-fraction probation (`len/4`) is too short in 128-slot buckets for web_0's late second touches. This held in every lineage until a depth floor was added.\n- Per-op-time and metadata-size have gone unmeasured in all five cycles. Every latency or cost claim is still untested, because the metric is missing from the fitness vector itself (a harness gap).\n\n## Open questions\n\n- How high can the small-bucket probation floor go before hm_0 starts losing main capacity?\n- Does a larger probation fraction in 1024-slot buckets help web_0 at the large size, or does it weaken the filtering gained in cycle 0004?\n- Can set dueling on `small_len` (leader buckets at different depths) pick the right depth per workload without a fixed rule?\n- What explains the rest of web_0's small-size gap, given that depth only partly closed it and admission strength made it worse?\n- What do GCLOCK sweeps, mid-vector inserts and miss-path ring scans actually cost once the harness reports per-op-time?\n\n## Per-policy notes\n\n- **LRU + LIP/BIP (cycles 0001–0004):** It won early but is now stuck. Every change after 0002 traded hm_0 (wants a large protected region) against web_0 (wants deep probation). The only item left worth trying is an asymmetric-PSEL fix; otherwise this line can be retired.\n- **CLOCK → GCLOCK (cycles 0001–0003):** Hit-rate was even or better than its parent every time, and the gains were largest on hm_0 in large buckets. Ghost warm admission added nothing here.\n- **GCLOCK main + probation FIFO (cycles 0004–0005, current lineage):** This is the first design to help both traces. Main is frequency-ranked and its sweep runs only on promotion, so it needs fewer slots than an LRU protected segment, and the probation/main trade-off is not zero-sum. Keep the counter reset to 0 on promotion, since the position behind the hand already gives a full rotation of grace. The cycle 0005 parent adds an absolute depth floor, which moved small buckets only. Large-bucket cells have not changed since cycle 0004.\n"
}
```

### 15:05:20 chat

I rebuilt `cross-cycle-insights.md` from scratch using only the ledger digest for cycles 0001–0005, with the four required sections in order.

The main points across the five cycles:

- **Recurring wins:**
  - Placing new items below reused ones (cold insertion) is the most reliable lever. It won in cycle 0001 on LRU and again in cycle 0004 on GCLOCK with a probation ring, where it helped both traces with no trade-off.
  - Deeper probation helps small buckets.
  - GCLOCK-style recency never lost hit-rate against exact LRU. It is the only line that has improved the large buckets.
  - A hit during probation is the only promotion signal that has held up.
- **Recurring failures:**
  - Making it easier to enter or stay in main space (ghost admission, keeping a counter on promotion, weaker or graded promotion) kept causing churn, mostly on web_0. Ghost admission is retired.
  - The self-adjusting PSEL feedback in the LRU line failed twice.
  - In LRU, the probation floor and occasional MRU insertion don't stack, because they split the same bucket space.
  - A fixed quarter-of-bucket probation is too short in 128-slot buckets.
- **Measurement gap:** latency (per-op-time) and metadata size have never been measured in any cycle. Every cost claim is still untested, because the harness's fitness vector doesn't include them.
- **Open questions:**
  - How far the small-bucket probation floor can go before hm_0 suffers.
  - Whether deeper probation helps web_0 in large buckets.
  - Whether set dueling on probation depth can choose the depth per workload.
  - What explains the rest of web_0's small-bucket gap.
  - What the operations actually cost once latency is measured.
- **Per-policy notes:**
  - LRU with cold/occasional-MRU insertion is stuck trading one trace against the other.
  - Plain CLOCK/GCLOCK was steady and scales well.
  - GCLOCK with a probation ring is the current lineage. Its large-bucket results haven't changed since cycle 0004.


---

- finished: 2026-10-28T15:05:21+00:00
- duration: 25.57s
- status: ok
- input_tokens: 4
- output_tokens: 2226
- cache_read_tokens: 39042
- cache_write_tokens: 10622
- cost_usd: 0.13732
