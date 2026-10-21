# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-21T05:14:24+00:00

## timeline

### 05:14:37 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Low protection for new keys on insert is the main hit-rate lever on workload-a. It has won in all three cycles (0001–0003) and helped both the shift-based LRU line and the in-place CLOCK line. New keys must earn protection with a second hit.\n- Making the newcomer filter stronger has kept paying off. Insertion moved from the midpoint to 1/4 and then to 1/8, and the CLOCK hand now rests on the newcomer. Even a grace window of about one to two misses has not hurt hit rate on this trace.\n- Exact LRU ordering inside a bucket is not needed. In-place CLOCK/GCLOCK beat the baseline in every cycle, and in Cycle 0003 it nearly matched deep-insertion LRU once its newcomer filter was equally strong.\n- Shrinking the item record had no effect on hit rate (Cycle 0001), so the per-item wall-clock timestamp can safely be removed.\n\n## Recurring failures\n\n- Extra protection beyond the newcomer filter keeps adding little or nothing. Sending ghost hits straight to MRU lost hit rate (Cycle 0002). Picking victims by frequency at the eviction end was within noise (Cycle 0003). One recurrence does not predict a near-term hit on this trace.\n- Frequency counters inside the bucket (GCLOCK) helped plain CLOCK a little (Cycle 0002), but did not close the gap. The gap closed only once the hand placement filtered newcomers.\n- Per-op-time and metadata-size have not been recorded in any cycle. As a result, every latency or size hypothesis is still untested, and the main ranking tradeoff cannot be settled.\n- Each depth step gains less than the one before it: 1/4 to 1/8 gained less than midpoint to 1/4. Hit-rate tuning along the depth axis is close to its limit.\n\n## Open questions\n\n- Does pure LIP (index 0) improve on 1/8, or has the depth curve already peaked?\n- Once per-op-time is recorded, does in-place GCLOCK with the hand resting on the newcomer beat deep-insertion LRU overall? The LRU line pays more element shifts on every miss as depth grows.\n- Is the aggressive newcomer filter over-fitted to workload-a? Every result so far comes from one trace, with no checks at other cache sizes or reuse distances.\n- Could frequency help on the insertion or promotion side, for example by setting insert depth from a key's count, even though it failed on the eviction side?\n- Does a shared global atomic clock (from the smaller-item variant) hurt multi-threaded throughput?\n\n## Per-policy notes\n\n- **Shift-based LRU with an insert depth (midpoint, 1/4, 1/8):** won hit-rate in every cycle, with each gain smaller than the last. Its miss-path cost (shifting part of the bucket on every miss) grows with depth and has never been measured.\n- **CLOCK / GCLOCK (in-place):** beat the baseline in all three cycles. The single bit lost useful signal, and the saturating counter helped. The biggest gain came from leaving the hand on the new key. This line has the strongest latency case, but that case is unverified.\n- **Ghost list with MRU admission:** hurt hit rate. It has been dropped and should come back only in a much more conservative form, such as a partial promotion or a small, recent ghost ring.\n- **Frequency-aware eviction window on deep-insertion LRU:** gain was within noise. Drop it.\n- **Logical u32 stamps / smaller Item:** hit rate unchanged. Its size benefit still needs metadata-size to be measured. It could be combined with the CLOCK line.\n"
}
```

### 05:14:43 chat

I wrote `cross-cycle-insights.md` from the ledger for cycles 0001–0003, using the four required sections in order. The main patterns:

- **What keeps winning:** making new keys earn protection with a second hit is the main hit-rate lever on workload-a. It won in all three cycles, in both the shift-based LRU line and the in-place CLOCK line.
- **Getting close to the limit:** moving insertion deeper (midpoint, then 1/4, then 1/8) helped each time, but each step gained less than the one before.
- **What keeps failing:** giving extra protection beyond that new-key filter (ghost-list promotion, frequency-based eviction) added little or nothing.
- **Biggest gap:** per-op-time and metadata-size have never been recorded. So no cycle has tested any latency or size claim, and the choice between the two leading designs is still open.
- **Open questions:** whether inserting at the eviction end (pure LIP) beats 1/8, whether the aggressive filter only suits this one trace, using frequency on the insertion side instead, and whether a shared global atomic clock hurts multi-threaded throughput.
- **Per-policy notes:** one short block each for the shift-based LRU line, CLOCK/GCLOCK, the ghost list, the frequency-aware eviction window, and the smaller item record.

The file is in this cycle's `cross-cycle/` output directory.


---

- finished: 2026-10-21T05:14:43+00:00
- duration: 19.42s
- status: ok
- input_tokens: 4
- output_tokens: 1836
- cache_read_tokens: 35599
- cache_write_tokens: 8303
- cost_usd: 0.11069
