# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:37:26+00:00

## timeline

### 09:37:47 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through Cycle 0009)\n\n## Recurring wins\n\n- **A strict newcomer filter is the main lever on workload-a (cycles 1–4).** New keys enter with little protection and must prove reuse before they are promoted. This held for midpoint and deep insertion on LRU and for cleared-bit or hand-on-newcomer on CLOCK. Filtering out one-off keys matters more than exact recency order.\n- **There is a grace optimum, and it sits just short of the extreme.** On LRU, deeper insertion helped each step down to 1/8, then pure LIP lost. Newcomers need slightly more than one miss of grace.\n- **Capped ghost admission works on both lines (cycles 4–5).** A small, recent ring of recently evicted keys that grants only a small reward (counter 1 on CLOCK, a slightly deeper slot on LRU) gains hit rate. Both the reward and the trigger must stay small and selective.\n- **Cost-only changes land cleanly (cycles 3, 7–9).** Fingerprint ghost rings, dropping the wall-clock stamp, narrowing the counter, and a lower ceiling all kept hit rate exactly or nearly. The ghost ring is used only as a membership test, and the winning paths never read the removed state.\n- **The base has converged.** CLOCK/GCLOCK with the hand resting on the newcomer, a +1 hit step, a ceiling of 3, an unfiltered 4-entry fingerprint ghost ring, and the compact item (cycle/0009/v002). LRU at 1/8 depth with ghosts entering at 1/4 is the reference line.\n\n## Recurring failures\n\n- **Large rewards for weak signals.** Full MRU promotion on a ghost hit (cycle 2), a larger first-hit step (cycle 8), and a broad miss-frequency sketch that saturated and granted counter 1 to most newcomers (cycle 7). Each one weakened the newcomer filter and lost hit rate.\n- **Frequency beyond the filter.** Frequency-based victim choice at the eviction end (cycle 3), SLRU-style promotion (cycle 4) and sketch admission (cycle 7) gave nothing or lost. Once the filter removes one-hit keys, little frequency signal is left.\n- **Adaptive leniency duels (cycles 5–6).** The self-referential version never chose leniency. The DIP-style version still lost, and the lenient leaders cost hits on their own. Newcomer leniency is closed on workload-a.\n- **Ghost-ring admission filters.** Excluding never-hit keys (LRU, cycle 6) and excluding hit keys (CLOCK, cycle 8) both lost. The unfiltered ring is already complete.\n- **Tuning single knobs on workload-a (cycles 5–8).** Every knob has now been probed in both directions and peaks at its current setting: insert depth, ring size, ring admission, ghost-hit depth, newcomer grace, hit step and ceiling. No cycle since 4 has gained hit rate.\n- **Multi-bucket or alternate placement (cycle 9).** The slot-legality invariant forbids it however it is implemented.\n- **The evaluation gap that never closed.** Every cycle's continuation asked for per-op-time and metadata-size to be recorded, and no cycle has delivered them. As a result, all cost-side wins and trade-offs are still invisible.\n\n## Open questions\n\n- **Cost metrics.** Once per-op-time and metadata-size are recorded:\n  - Does CLOCK+ghost clearly beat LRU+ghost, given that LRU shifts elements on every miss?\n  - Is ceiling 2 or a 2–3 entry fingerprint ring worth its small hit-rate loss?\n  - Does fingerprint hashing on the miss path outweigh its memory saving?\n  - Does the shared atomic stamp contend under multithreaded load?\n- **Generalisation.** Do the strict filter, the narrow ghost ring and the one-miss grace hold on a second trace or cache size, especially one with longer reuse distances? Are the small losses from cycles 6–8 above noise?\n- **Bucket indexing.** Can a better-mixing bucket-index hash that keeps each key in a single bucket, and so passes the invariant, reduce conflict misses?\n- **Compaction.** Can the CLOCK line's item or bucket metadata be compacted further, given the counter's tiny range?\n- **Bounded-error frequency admission.** Would a frequency trigger with a bounded false-positive rate ever add anything over the exact 4-entry ring? The trend so far says probably not.\n\n## Per-policy notes\n\n- **CLOCK/GCLOCK (main line).**\n  - Hits are applied in place, with no shifting.\n  - The gap to deep LRU came from where the hand rests, not from frequency (cycle 3). Capped ghost admission then put this line in front (cycle 4).\n  - Settled shape: hand-on-newcomer, +1 hit step, ceiling 3, 4-entry fingerprint ghost ring, compact item.\n  - Ring size has been bracketed: both 2 and 16 lost.\n  - Long-reuse keys dilute the ring on this line.\n- **LRU with insertion depth (reference line).**\n  - Depth 1/8 is the optimum. Pure LIP regresses.\n  - Ghosts that enter at 1/4 give a small gain. Entering at 3/8 gives nothing more.\n  - Aged-out keys that were hit at some point are exactly the population the ghost ring should rescue, so filtering them out hurts.\n  - The fingerprint ring carries over unchanged.\n  - Every miss pays a shift cost, which has never been measured.\n- **Classic LRU (baseline).**\n  - Exact recency order is unnecessary: a CLOCK approximation beat it.\n  - The per-item wall-clock timestamp is never read on the eval path, so dropping it is safe.\n- **Ghost rings (across lines).**\n  - The representation (a 16-bit fingerprint membership test) transfers between lines.\n  - Admission lessons do not transfer, because each line's ring captures a different population.\n"
}
```

### 09:37:54 thinking



### 09:37:54 chat

I wrote `cross-cycle-insights.md` in the cycle 0009 cross-cycle directory. It uses the four required sections in order and draws only on the ledger, with no numbers added.

What it records across cycles 1–9:

- **What keeps working:** the strongest lever is making new keys prove reuse before they get protection. The best setting stops just short of the extreme: insertion depth 1/8 beat pure LIP. A small ghost ring that gives only a small reward helped on both the CLOCK and LRU lines. Changes that only cut cost, like fingerprint rings and a smaller item record, kept hit rate unchanged. The current base is cycle/0009/v002.
- **What keeps failing:**
  - giving big rewards to weak signals;
  - adding frequency on top of the newcomer filter;
  - adaptive leniency duels;
  - filters on which keys enter the ghost ring;
  - single-knob tweaks on workload-a (every knob is now tested on both sides and peaks at its current setting);
  - placing keys in a second bucket, which the slot-legality invariant forbids.
- **The main gap:** every cycle asked for per-op-time and metadata-size to be recorded, and no cycle got them. That is why none of the cost savings show up in the rankings.
- **Open questions:**
  - which line wins once costs are measured;
  - whether the results hold on a second trace or cache size;
  - whether a better bucket-index hash that keeps each key in one bucket would help;
  - whether the metadata can be compacted further.
- **Per-policy notes** cover CLOCK/GCLOCK (the main line), LRU with insertion depth (the reference line), the classic LRU baseline, and how ghost rings behave on each line.


---

- finished: 2026-10-21T09:37:55+00:00
- duration: 29.09s
- status: ok
- input_tokens: 4
- output_tokens: 2811
- cache_read_tokens: 41964
- cache_write_tokens: 15512
- cost_usd: 0.189772
