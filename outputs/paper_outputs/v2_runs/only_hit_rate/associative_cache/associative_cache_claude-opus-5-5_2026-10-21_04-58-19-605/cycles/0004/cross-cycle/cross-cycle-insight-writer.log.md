# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-21T05:19:13+00:00

## timeline

### 05:19:29 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- On workload-a, filtering newcomers is the main lever (cycles 1–4). New keys go in with low protection, either at a deep insert position or with the CLOCK bit cleared, and earn protection only through a later hit. Every cycle's winner either strengthened or refined this filter.\n- The filter works on both policy lines: shift-based LRU with deep insertion, and in-place CLOCK/GCLOCK. Exact recency order inside a bucket is not needed to get the gain.\n- Make the filter strict, then add selective grace. Once the strict filter has peaked, extra grace helps only when it goes to keys with evidence of short reuse, and only when that grace is capped. The CLOCK ghost ring that admits returning keys with counter 1 won cycle 4.\n- Removing the per-item wall-clock timestamp (logical stamps) did not change hit rate. It is safe to drop for hit rate, though the size benefit has not been measured.\n\n## Recurring failures\n\n- Uncapped rewards for returning keys. When a ghost hit sent the key to MRU (cycle 2), hit rate dropped because the reward weakened the newcomer filter. How big the reward is matters as much as the signal that triggers it.\n- Frequency-based protection beyond the newcomer filter. Three cycles in a row it gave little or nothing: victim choice by frequency at the eviction end (cycle 3), SLRU-style two-step promotion (cycle 4), and GCLOCK counters, which only helped CLOCK relative to plain CLOCK (cycle 2). Once the filter removes keys that are hit only once, frequency has little left to separate. Caveat: the cycle 4 promotion test was weak because the demotion flag was never reset.\n- Pushing insert depth all the way to pure LIP (index 0) lost hit rate. Depth returns shrank from 1/2 to 1/4 to 1/8, then turned negative. Newcomers need slightly more than one miss of grace.\n- Evaluation gap (process failure, all 4 cycles): per-op-time and metadata-size were never recorded. Every latency or size claim is untested, including the miss-path costs of shifts, ghost scans and hand sweeps.\n\n## Open questions\n\n- Latency and memory: once per-op-time and metadata-size are measured, which comes out ahead overall, CLOCK/GCLOCK + ghost (no shifts, but a ghost scan on each miss and extra keys per bucket) or the 1/8-depth LRU line (shifts on each miss)?\n- Generalisation: does the aggressive, about one-miss filter, and the ghost gain, hold on traces with longer reuse distances or at other cache sizes? Every result so far comes from workload-a at a single size.\n- Ghost ring size: is the best size smaller than 4, larger, or scaled to the bucket?\n- Does capped ghost admission carry over to the 1/8 LRU line, for example by inserting a ghost hit one step deeper than a normal newcomer?\n- Does a shared global atomic clock for logical stamps hurt multi-threaded throughput?\n\n## Per-policy notes\n\n- LRU with insertion position (midpoint, then deeper insertion): won cycles 1–3. Insert depth 1/8 is the tested optimum, so depth tuning is done. Each miss shifts about half or more of the bucket, and that cost has not been measured.\n- CLOCK / GCLOCK: improved its hit rate every cycle, and beat baseline LRU even with latency as its stated goal. Its gap to the LRU line came from where the hand rests, not from frequency. Leaving the hand on the newcomer brought it close to 1/8 LRU. Adding a capped ghost ring made it the current leader. It is the strongest candidate to become the main line once latency counts.\n- Ghost / history-based admission: helps only when the reward is capped (an extra sweep pass) and fails with full MRU promotion. So far it has been tested only on the CLOCK line.\n- SLRU / frequency promotion: about noise on top of the 1/8 filter. Low priority unless it is retried cheaply with correct flag resets.\n- Item layout (logical u32 stamps): neutral for hit rate. It could be combined with any line, but its size win has not been measured.\n"
}
```

### 05:19:36 chat

I've written `cross-cycle-insights.md` from the four cycles in the ledger. It has the four required sections in order, and I built it fresh from the ledger without reading the previous version.

**Recurring wins:**
- On workload-a, the strongest lever is making new keys prove they're reused before they get protection. Every cycle's winner made that filter stronger or refined it, and it works for both the LRU and the CLOCK policies.
- Once the filter was as strict as it could usefully get, the next gain came from giving a limited amount of extra time only to keys that recently came back. In cycle 4 that was a small per-bucket record of recently evicted keys on the CLOCK line ("ghost ring").

**Recurring failures:**
- Big rewards for returning keys hurt. In cycle 2, sending them straight to the most-protected slot lost hit rate.
- Protection based on how often a key is hit added little or nothing in three cycles running. The cycle 4 test of this was weak because a flag was never reset.
- Putting new keys at the very eviction end lost hit rate. Inserting 1/8 of the way up the bucket is the best depth tested.
- Speed per operation and memory per item haven't been measured in any of the four cycles. Every latency or size claim is still untested.

**Open questions:**
- Which policy line comes out ahead once speed and memory are measured?
- Do the results hold on other traces or cache sizes? Everything so far comes from one trace at one size.
- What size of ghost ring works best?
- Does the capped ghost reward also help the LRU line?
- Does the shared global clock used for logical timestamps slow down multi-threaded use?

**Per-policy notes:** The file has one entry each for LRU with insert depth, CLOCK/GCLOCK, ghost admission, SLRU-style promotion, and the smaller item record. CLOCK with the ghost ring leads on hit rate now and is the strongest candidate to be the main line once speed is measured.


---

- finished: 2026-10-21T05:19:36+00:00
- duration: 23.01s
- status: ok
- input_tokens: 4
- output_tokens: 2200
- cache_read_tokens: 36608
- cache_write_tokens: 9493
- cost_usd: 0.127793
