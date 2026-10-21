# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:11:21+00:00

## timeline

### 09:11:40 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Low protection for newcomers is the main lever on workload-a (all 5 cycles).** New keys enter near the eviction end (or with a cleared or low counter) and earn protection only when they are reused. This beats keeping exact recency order. It has worked on both shift-based LRU and in-place CLOCK/GCLOCK.\n- **The filter should be strict but not total.** Hit rate rose as the filter got stricter, up to about one slot (one miss) of grace: 1/8 depth on LRU, and the hand resting on the newcomer on GCLOCK. Pure LIP went past the peak.\n- **Capped ghost admission helps.** A small, recent ghost ring that gives a returning key only a little extra grace improved hit rate on GCLOCK (Cycle 0004) and, by a smaller margin, on the 1/8 LRU line (Cycle 0005). It carries over between policy lines.\n- **The size of the reward matters as much as the signal that triggers it.** The same signal (a ghost hit or a frequency count) helps when its reward is small and bounded, and hurts when the reward is a full promotion.\n\n## Recurring failures\n\n- **Frequency-based protection beyond the newcomer filter (Cycles 0002–0004).** This covers frequency-aware victim choice at the eviction end and SLRU-style promotion by frequency. The gains were at or near noise level, because the strict newcomer filter already removes keys that are used only once. GCLOCK counters helped plain CLOCK only before its newcomer filter was fixed.\n- **Generous ghost rewards.** Sending ghost hits straight to MRU (Cycle 0002) lowered hit rate. Growing the ghost ring (from 4 to 16) diluted the signal and pushed out valuable residents (Cycle 0005).\n- **Overshooting on a tuning axis.** Insert depth (pure LIP) and ghost ring size both have an interior peak. Pushing either one to the extreme regressed.\n- **Adaptive controllers that learn from their own outcome.** The strict-versus-lenient duel locked itself into strict mode, so leniency was never actually tested (Cycle 0005).\n- **The evaluation gap that never closed.** Per-op-time and metadata-size have not been recorded in any cycle. Every claim about latency or memory is untested, even though most winning variants added miss-path work: shifts, ghost scans, hand sweeps, per-bucket state.\n\n## Open questions\n\n- Once latency and memory are measured, which line should be the primary one: in-place GCLOCK with ghosts, or 1/8-depth LRU with ghosts? The ranking is currently based on hit rate alone.\n- Do the strict filter and the small ghost ring generalise to other traces, longer reuse distances, or other cache sizes? Or are they over-fitted to workload-a at one size?\n- Would a smaller ghost ring (2–3 entries) match or beat 4, while also cutting miss-path and memory cost?\n- Would a properly built set-dueling controller (DIP-style leader buckets with an unbiased negative signal) ever choose leniency?\n- Are the latest small margins above run-to-run noise? Hit rate on workload-a looks close to saturated.\n\n## Per-policy notes\n\n- **Shift-based LRU (insertion depth):** depth 1/2 → 1/4 → 1/8 gave shrinking gains, and index 0 regressed, so depth tuning is done. Capped ghost admission (a ghost hit enters slightly deeper than a normal newcomer) adds a small gain. Each miss pays for element shifts, a cost that has not been measured.\n- **CLOCK / GCLOCK:** cleared bits on insert beat LRU even though CLOCK was designed for latency. Saturating counters helped over a single bit. The real gap to the LRU line was where the hand rests, not frequency. With the hand resting on the newcomer plus a 4-entry capped ghost ring, this is the strongest line by hit rate. Its no-shift hits may give it an edge on latency once latency is measured.\n- **Item metadata:** replacing the wall-clock timestamp with a logical stamp kept hit rate unchanged. Shrinking the item record looks safe, but no size or throughput data exists yet, including any contention on a shared atomic clock.\n"
}
```

### 09:11:47 chat

I wrote `cross-cycle-insights.md` from the ledger alone. Every ranking in it rests on hit rate on one trace (workload-a) at one cache size, because per-op-time and metadata-size were not recorded in any of the five cycles.

- **Recurring wins:**
  - The strongest pattern, in every cycle, is giving new keys low protection so they must be reused before they're kept.
  - The best strictness is about one miss of grace. Inserting at 1/8 depth hit that on the LRU line, and so did leaving the hand on the new key on the CLOCK line. Pure LIP went too far.
  - Ghost admission helps only when the reward for a returning key is small and capped. It helped on both the CLOCK and LRU lines.
- **Recurring failures:**
  - Extra protection based on how often a key is used added little or nothing on top of the newcomer filter.
  - Generous ghost rewards (straight to MRU) and a larger ghost ring both lost hit rate.
  - Pushing a tuning setting to its extreme regressed every time. Both insert depth and ghost ring size peak in the middle.
  - The adaptive controller ended up feeding on its own results, so the lenient option was never actually tested.
  - The missing latency and memory measurements are listed here as well.
- **Open questions:**
  - Which line should lead once latency and memory are counted?
  - Do the results hold on another trace or cache size?
  - Would a 2–3 entry ghost ring do as well as 4?
  - Would a properly built set-dueling controller ever pick leniency?
  - Are the latest small margins bigger than run-to-run noise?
- **Per-policy notes:**
  - **Shift-based LRU:** depth tuning is finished. Capped ghost admission adds a small gain.
  - **CLOCK / GCLOCK:** with the hand resting on the new key plus a 4-entry capped ghost ring, this is the strongest line on hit rate.
  - **Item records:** shrinking them looks safe for hit rate, but their size and throughput haven't been measured.


---

- finished: 2026-10-21T09:11:47+00:00
- duration: 25.53s
- status: ok
- input_tokens: 4
- output_tokens: 2261
- cache_read_tokens: 37000
- cache_write_tokens: 11163
- cost_usd: 0.142552
