# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-21T05:09:13+00:00

## timeline

### 05:09:26 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Low protection on insert is the strongest lever on workload-a, and it has held up in both cycles. New keys enter weakly protected (midpoint or deeper in the LRU order, or with the CLOCK bit cleared) and earn protection only after a second hit. This filters out one-off keys. Every hit-rate winner so far uses it.\n- Making the filter stronger keeps helping. Moving insertion from the midpoint (cycle 1) to deeper toward the eviction end (cycle 2) raised hit rate again.\n- Exact recency order inside a bucket is not required. CLOCK, and GCLOCK after it, beat the baseline hit rate without strict LRU order, so cheaper in-place hit paths are an option.\n- Frequency signal adds value over a single recency bit. A small saturating counter (GCLOCK) improved on plain CLOCK.\n\n## Recurring failures\n\n- Anything that weakens the newcomer filter loses hit rate. Ghost-list admission that sends returning keys straight to MRU did worse than its parent. On this trace, one recurrence at a moderate reuse distance does not predict near-term reuse.\n- Per-item wall-clock timestamps do nothing for hit rate on the eval path. Cycle 1 recommended dropping wall-clock ordering work.\n- The evaluation gap has persisted for two cycles. Per-op-time and metadata-size have never been recorded. So every latency or size claim is untested, including CLOCK's cheaper hits, the smaller Item record and every added miss-path cost (Vec shifts, ghost hashing and scans, longer hand sweeps). This is the most important process failure to fix.\n\n## Open questions\n\n- Where is the best insert depth? Bracket between about 1/8 and pure LIP (index 0) to see whether hit rate peaks before index 0.\n- Can the CLOCK/GCLOCK line match deep insertion's hit rate? Options are giving newcomers protection below that of a single hit, or raising the counter cap. If it can, it would win on hit rate and on (still unmeasured) latency.\n- What is the true per-op-time cost of insertion that shifts elements on every miss, compared with CLOCK sweeps? This can only be answered once timing is collected.\n- Would a shared global atomic logical clock hurt multi-threaded throughput?\n- Could a much more conservative ghost policy recover any gain, such as partial promotion or a smaller ring holding only recent entries? This is low priority.\n\n## Per-policy notes\n\n- **LRU with insertion position (midpoint, then deeper):** best hit rate in both cycles. Its miss-path cost of shifting about half the bucket or more on every miss is still unmeasured.\n- **CLOCK (bit cleared on insert):** beat the baseline hit rate even though it was written as a latency play. It trailed the insertion-position line.\n- **GCLOCK (saturating counter):** improved on CLOCK but still did not reach deep insertion. It is the natural base for a hybrid with in-place hits and stronger newcomer filtering.\n- **Ghost list with MRU promotion:** a hit-rate regression. Drop it unless it comes back in a much more cautious form.\n- **Compact Item (logical u32 stamps):** left hit rate unchanged, as predicted. Its size benefit has not been measured, and it is worth stacking onto the winning lines.\n"
}
```

### 05:09:31 chat

I wrote `cross-cycle-insights.md` from the ledger for cycles 0001–0002, using the four required sections in order.

**Main takeaways:**
- **What keeps winning:** new keys go in with low protection and have to earn it with a second hit. That's either insertion at the midpoint or deeper, or CLOCK with the bit cleared on insert. Pushing insertion deeper helped again in cycle 2. Exact recency order doesn't matter much. A frequency counter (GCLOCK) improved on plain CLOCK.
- **What keeps failing:** anything that weakens that newcomer filter. The ghost list that sends returning keys straight to the most-protected position lost hit rate. Also, per-op-time and metadata-size have not been recorded in either cycle, so every claim about latency or record size is still untested. This is the most important thing to fix.
- **Open questions:** the best insert depth, somewhere between about 1/8 and index 0. Whether CLOCK/GCLOCK can match deep insertion on hit rate. The real per-op cost on misses for each approach. Whether a shared atomic clock hurts multi-threaded throughput. Whether a much more cautious ghost policy could help (low priority).
- **Per-policy notes** cover the deep-insertion LRU line, CLOCK, GCLOCK, the ghost list and the smaller item record. The smaller record kept hit rate unchanged, but its size benefit hasn't been measured.

All comparisons are qualitative; I didn't add any numbers that aren't in the ledger.


---

- finished: 2026-10-21T05:09:31+00:00
- duration: 18.72s
- status: ok
- input_tokens: 4
- output_tokens: 1771
- cache_read_tokens: 34710
- cache_write_tokens: 7300
- cost_usd: 0.101099
