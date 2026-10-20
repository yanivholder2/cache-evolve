# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-19T12:13:14+00:00

## timeline

### 12:13:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Leaving strict LRU order pays off. Every policy that stopped relinking on each hit or protected reused keys from scans gained hit-rate on workload-a. That covers SLRU, CLOCK, BIP, GCLOCK and SIEVE.\n- A hit path with no relinking (hand-based policies) keeps get latency low in every cycle where it was tried.\n- Node compaction is a reliable metadata lever: `u32` index links, fewer links (CLOCK, singly-linked SIEVE), and policy bits stored in padding bytes. Once the hit path stops relinking, compaction no longer speeds things up; it only saves metadata.\n- Rewriting the representation while keeping the eviction order identical is the safest way to improve. It has never cost hit-rate.\n- Simple beats layered: SIEVE (one queue, one bit, quick demotion of new keys, lazy promotion of reused keys) matched or beat the segmented policies.\n\n## Recurring failures\n\n- Adding more in-queue reuse signal has not helped. Deeper counters (GCLOCK, the 2-level SIEVE counter) cost put-tail latency and, on SIEVE, hit-rate as well.\n- Scan defenses don't add up. Putting BIP head insertion inside SLRU probation shrank the window new keys have to earn a second touch, and hit-rate dropped.\n- Cutting the sweep short (capping the hand's steps) evicts runs of visited nodes that carry real reuse. It cost hit-rate and did not fix the put tail.\n- \"Labeled Winner\" is a weak signal. Several winners had flat or negative results on the main dimensions, so judge each one by what actually moved.\n\n## Open questions\n\n- Does the ranking generalize? After three cycles every result still comes from workload-a at size 2048. This is the top unaddressed risk.\n- Can history of evicted keys (ghost entries, S3-FIFO small/main/ghost, ARC/LIRS-style tracking of keys no longer in the cache) add hit-rate where in-queue counters could not?\n- What actually sets put p99? Sweep length has been ruled out. Suspects are the per-miss HashMap remove/insert, the key clone and cold-node touches.\n- Are SLRU's probation sizing or an admission filter still worth testing, given that changing where new keys are inserted failed?\n\n## Per-policy notes\n\n- LRU (baseline): beaten on every axis by every alternative tried. Reordering tweaks have been dropped.\n- SLRU: second-touch promotion gives solid hit-rate, and `u32` links shrink the node. BIP inside probation hurt. Any future change should target the segment split or admission, not where new keys are inserted.\n- CLOCK / GCLOCK: the reference-bit ring is cheap and gained hit-rate unexpectedly. GCLOCK's deeper credit buys hit-rate at a visibly worse put tail.\n- BIP: works as a standalone insertion policy with an early speed gain whose reliability is uncertain. It conflicts with probation-based schemes.\n- SIEVE: the current best lineage and parent. Singly-linked SIEVE (the hand carries its predecessor) is the most compact version with unchanged behavior. Counter-depth and sweep-cap variants are exhausted. Next steps are ghost/history signals and work on the HashMap path on misses.\n"
}
```

### 12:13:34 chat

I wrote `cross-cycle-insights.md` from the three-cycle ledger, using the four required sections in order.

The biggest risk it records is that every result so far comes from one trace (workload-a) at one cache size (2048). Nothing has yet tested whether the rankings hold elsewhere, even though each cycle's notes asked for it.

- **Recurring wins:**
  - Moving away from strict LRU order has paid off every time.
  - Policies that don't move entries on a cache hit (CLOCK, SIEVE) keep reads fast.
  - Smaller nodes reliably cut metadata, but only metadata once a hit no longer moves entries.
  - Changing the data layout while keeping the same eviction order has never cost hit-rate.
  - SIEVE, the simplest policy tried, matched or beat the multi-segment ones.
- **Recurring failures:**
  - Adding more reuse tracking inside the queue (deeper counters) costs latency and sometimes hit-rate.
  - Stacking scan defenses doesn't add up; BIP inside SLRU lost hit-rate.
  - Capping the eviction sweep throws away useful entries and didn't fix slow inserts.
  - Several "Winner" variants actually came out flat or worse on the main measures.
- **Open questions:**
  - Whether results hold on other traces and cache sizes.
  - Whether remembering recently evicted keys (ghost entries, S3-FIFO, ARC/LIRS) can raise hit-rate.
  - What actually causes slow inserts, now that sweep length is ruled out. The hash-map work on each miss is the main suspect.
  - Whether changing SLRU's segment sizes or adding an admission filter is still worth testing.
- **Per-policy notes:** separate notes for LRU, SLRU, CLOCK/GCLOCK, BIP and SIEVE. SIEVE is the current parent, and its next steps are tracking evicted keys and speeding up the hash-map work on misses.


---

- finished: 2026-10-19T12:13:35+00:00
- duration: 20.31s
- status: ok
- input_tokens: 4
- output_tokens: 1927
- cache_read_tokens: 35651
- cache_write_tokens: 8244
- cost_usd: 0.111638
