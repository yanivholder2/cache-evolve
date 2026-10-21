# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:28:40+00:00

## timeline

### 09:28:58 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Strict newcomer filter is the dominant lever (cycles 1–4).** New keys enter with low protection and earn it only on a second hit. This improved hit rate on both shift-based LRU and in-place CLOCK. Stricter kept paying off until about one miss of grace; pure LIP went too far.\n- **Capped ghost admission (cycles 4–5).** A small recent-eviction ring that gives returning keys a single extra unit of protection helped on both the CLOCK and LRU lines. Cycles 2, 4 and 8 point to the same rule: the size of the reward matters as much as the signal, so rewards must stay small for ghost returns and for first hits alike.\n- **Ghost ring as a membership test (cycle 7).** Storing compact fingerprints instead of full keys left hit rate unchanged. This is the only cost-reducing change confirmed so far on the settled design.\n- **Settled best shape: cycle/0007/v001.** CLOCK/GCLOCK with the hand resting on the newcomer, a +1 hit step, a counter ceiling of 3, and a 4-entry unfiltered fingerprint ghost ring. Every knob on it has been probed in both directions and peaks at its current setting.\n\n## Recurring failures\n\n- **Frequency-based protection beyond the newcomer filter (cycles 3, 4, 7).** This covered victim choice at the eviction end, SLRU-style promotion, and a broad miss-frequency admission sketch. It added nothing, because the filter already removes keys used only once. The broad sketch actively hurt: it saturated and in effect switched the filter off. Signals that grant a reward must stay selective.\n- **Generous ghost rewards and wide rings (cycles 2, 5).** Promoting ghost hits to MRU, or growing the ring to cover longer reuse distances, diluted the signal and pushed out valuable residents.\n- **Ghost-ring admission filters (cycles 6, 8).** Excluding never-hit keys (on LRU) or hit keys (on CLOCK) both lost hit rate. The unfiltered ring is already complete.\n- **Newcomer-leniency duels (cycles 5, 6).** Self-referential controllers learned from their own outcome and stayed stuck in strict mode. Fixed DIP-style leaders removed that bias, but lenient grace still lost and the leaders cost hits. This thread is closed on workload-a.\n- **Single-knob hit-rate tweaks after cycle 5.** Three cycles in a row produced no gain. The workload-a hit-rate axis is saturated for this design family.\n- **Persistent evaluation gap (every cycle).** Per-op-time and metadata-size have never been recorded. Cost-saving variants (fingerprints, ceiling 2, smaller rings, cheaper hits) are invisible to the evaluation, and miss-path costs added by the winners have never been measured.\n\n## Open questions\n\n- Once per-op-time and metadata-size are measured, which shape wins: the CLOCK+ghost line, the 1/8 LRU+ghost line, a 3-entry fingerprint ring, or ceiling 2?\n- Does the strict one-miss filter plus a 4-entry ghost ring generalise to a second trace or cache size, especially one with longer reuse distances? Or is it over-fitted to workload-a?\n- Are the small losses and wins since cycle 5 above run-to-run noise?\n- Does fingerprinting work equally well on the LRU line's ghost ring?\n- Would a frequency-based admission signal with bounded false-positive error ever add anything beyond the exact ghost ring?\n\n## Per-policy notes\n\n- **Shift-based LRU (midpoint/deep insertion):** insert depth peaked at 1/8; pure LIP regressed. Ghost hits inserted at 1/4 depth help slightly, and deeper insertion (3/8) does not. Ghost admission filtering hurts. All knobs are bracketed, so keep this as a reference line only. Shifts on every miss carry an unmeasured latency cost.\n- **CLOCK / GCLOCK:** the gap to LRU closed once the hand rested on the newcomer (cycle 3). A capped ghost ring then gave it the lead (cycle 4). Ring size peaks near 4, with both 2 and 16 losing. Hit step +1 with ceiling 3 is optimal. In-place hits make it the likely latency winner, but that is unverified.\n- **Ghost mechanisms in general:** lessons do not transfer cleanly between lines, because each line's ring captures a different population (long-reuse keys on CLOCK, keys just past the bucket on LRU). Check each rule on each line separately.\n- **Wall-clock timestamps:** they have no effect on eviction decisions, so smaller item records (logical stamps) are safe for hit rate. A shared atomic clock's multi-threaded cost is untested.\n"
}
```

### 09:29:05 thinking



### 09:29:05 chat

I wrote `cross-cycle-insights.md` in the cycle 0008 cross-cycle output folder. It draws only on the eight-cycle ledger and keeps the four required sections in order.

- **Recurring wins:** the strongest lever is making new keys prove reuse before they get protection. It worked on both the LRU and CLOCK designs, with about one miss of grace as the best setting. Remembering recently evicted keys in a small ring (a ghost ring) and giving returning keys only a small extra reward also helped. Storing short fingerprints in that ring instead of full keys gave the same hit rate. The best design so far is cycle/0007/v001, and every setting on it has been tested in both directions.
- **Recurring failures:**
  - Adding frequency information on top of the newcomer filter never helped.
  - Large ghost rewards, larger rings, and filters on which keys enter the ring all lost hit rate.
  - Adaptive switching between strict and lenient newcomer handling lost even after its learning bias was fixed.
  - The last three cycles of hit-rate-only tweaks on workload-a gained nothing.
  - Per-op-time and metadata-size have never been recorded in any cycle, so cost-saving variants can't be told apart.
- **Open questions:** which design wins once costs are measured, whether the design works on a second trace or cache size, whether the recent small differences are above noise, and whether fingerprints work on the LRU line too.
- **Per-policy notes:** LRU is fully tuned (insert at 1/8 depth, ghost hits at 1/4) and should stay a reference line. CLOCK settles at a ring of about 4 entries, a hit step of 1 and a counter ceiling of 3. Ghost-ring lessons do not carry over cleanly between the two designs.


---

- finished: 2026-10-21T09:29:06+00:00
- duration: 25.84s
- status: ok
- input_tokens: 4
- output_tokens: 2422
- cache_read_tokens: 40868
- cache_write_tokens: 13980
- cost_usd: 0.169407
