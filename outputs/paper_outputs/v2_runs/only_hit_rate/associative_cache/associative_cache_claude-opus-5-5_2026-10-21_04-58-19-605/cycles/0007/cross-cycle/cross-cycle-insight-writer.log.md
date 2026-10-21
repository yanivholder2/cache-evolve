# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:22:31+00:00

## timeline

### 09:22:51 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **A strict newcomer filter is the main lever on workload-a (cycles 1–4).** A new key enters with low protection and earns protection only by being hit again. This improved hit rate on both shift-based LRU and in-place CLOCK. It peaked once newcomers had just over one miss of grace: 1/8 depth on LRU, hand-on-newcomer on GCLOCK.\n- **Capped ghost admission rescues returning keys (cycles 4–5).** A small, recent ghost ring gives a returning key one extra step of protection. That means counter 1 on CLOCK, or insertion slightly deeper on LRU, never a full MRU promotion. It helped on both lines. The size of the reward matters as much as the signal that triggers it.\n- **The ghost ring only needs to answer whether a key was evicted recently (cycle 7).** Storing 16-bit fingerprints instead of keys kept hit rate unchanged. This is the only change so far that lowers cost on the settled shape without losing hit rate.\n- **Exact LRU ordering is not required (cycles 1, 3).** In-place CLOCK/GCLOCK came close to deep-insertion LRU once its newcomer filter matched. Ordering by wall-clock timestamp adds nothing.\n- **Current base: cycle/0007/v001.** It combines GCLOCK, a strict hand-on-newcomer filter, and a 4-entry fingerprint ghost ring with a capped counter-1 bonus.\n\n## Recurring failures\n\n- **Uncapped or broad rewards.** Full MRU promotion on a ghost hit (cycle 2) hurt hit rate. So did a shared miss-frequency sketch that saturated and granted the bonus to most newcomers (cycle 7). The reward has to stay small, and its trigger has to stay selective.\n- **Frequency-based protection beyond the newcomer filter.** This was tried as victim choice at the eviction end (cycle 3), two-step SLRU-style promotion (cycle 4), and an admission sketch (cycle 7). It never paid off, because the strict filter already removes most keys that are used only once.\n- **Pushing any single knob past its peak.** Every knob has now been probed from both sides and peaked at its current setting: pure LIP insertion (cycle 4), ghost rings larger than 4 entries (cycle 5) or smaller (cycle 6), filtering which keys enter the ghost ring (cycle 6), and inserting ghost hits deeper than 1/4 on LRU (cycle 7).\n- **Adaptive leniency duels.** A self-referential per-bucket duel never chose the lenient mode (cycle 5). Fixed DIP-style leader buckets showed that leniency is simply worse here, and running lenient leaders costs hits (cycle 6). Newcomer leniency is closed on workload-a.\n- **Hit-rate-only evaluation.** Per-op-time and metadata-size have been missing for all 7 cycles. Cost-saving variants (CLOCK's in-place hits, the 2-entry ring, the fingerprint ring) can only tie, never win. Every miss-path addition is unpriced.\n- **Saturation.** Cycles 6 and 7 produced no hit-rate gain on workload-a. The current design family has hit its ceiling on this trace and size.\n\n## Open questions\n\n- Once per-op-time and metadata-size are recorded, which line should be the main one: CLOCK+ghost or 1/8 LRU+ghost? The suspicion is CLOCK, because it avoids shifting elements.\n- Is a 2- or 3-entry fingerprint ring worth its small hit-rate cost once memory and the miss-path scan are measured?\n- Does the strict one-miss filter, with its ghost bonus, generalise to a second trace or cache size? Traces with longer reuse distances are the main concern. Its tuning may be over-fitted to workload-a.\n- Can a frequency or admission signal with a false-positive rate bounded by design add anything? Or is the exact 4-entry ring already the complete admission signal?\n- Does a fingerprint ghost ring behave the same on the LRU line?\n\n## Per-policy notes\n\n- **Shift-based LRU (midpoint/deep insertion):** Insert depth was best at 1/8, and pure LIP regressed. Ghost hits do best when inserted at 1/4, and the curve is flat beyond that. Filtering out previously-hit keys from the ghost ring hurt, because on this line those keys are exactly the ones the ghost reward should rescue. SLRU-style promotion added nothing. Treat this as a reference line only, since all its knobs have been tuned. Each miss shifts about half a bucket, at a cost never measured.\n- **CLOCK/GCLOCK:** A saturating counter beats a single bit. Most of the gap to LRU came from where the hand rests (on the newcomer), not from frequency. A 4-entry ghost ring with a counter-1 bonus is the local optimum, and fingerprints work as well as full keys. Leniency duels and broad sketches hurt. This is the strongest line on hit rate, and it has the cheapest expected hit path.\n- **Ghost mechanisms in general:** Lessons do not carry over cleanly between lines. A rule that narrows the ring helps on CLOCK, but filtering ring admission hurts on LRU, because each line's ring selects a different population of keys.\n- **Baseline ClassicLRU:** The per-item wall-clock timestamp can be dropped with no effect on hit rate. A shared atomic clock may hurt multi-threaded throughput (not measured).\n"
}
```

### 09:22:58 chat

I've written `cross-cycle-insights.md` to the cycle 0007 cross-cycle folder, rebuilt from the ledger digest only. It uses the four required sections in order.

**Main takeaways across all seven cycles:**
- **What works:** A strict filter on new keys is the main lever on workload-a. A new key gets protection only after it is hit again, and the best setting gives it just over one miss of grace. On top of that, a small, recent ghost ring that gives returning keys one capped extra step of protection helps on both the CLOCK and LRU lines.
- **Suggested base:** `cycle/0007/v001`, which is GCLOCK with the strict filter and a 4-entry fingerprint ghost ring. Storing fingerprints instead of full keys kept hit rate the same, but its memory and miss-path savings don't show in the results because those dimensions aren't recorded.
- **What keeps failing:**
  - Giving returning keys a large reward, or granting the reward too widely.
  - Adding frequency-based protection on top of the new-key filter.
  - Adaptive duels that try to relax the filter.
  - Pushing any single setting further: every one has peaked at its current value.
- **Biggest gap:** Per-op-time and metadata-size have been missing for all seven cycles. Variants whose point is lower cost can only tie on hit rate, never win. Workload-a hit rate also looks maxed out for this design family: cycles 6 and 7 brought no gain.
- **Open questions:** Which line should be the main one once costs are measured, and whether the strict filter still works on a second trace or cache size, especially one with longer reuse distances.


---

- finished: 2026-10-21T09:22:58+00:00
- duration: 27.84s
- status: ok
- input_tokens: 4
- output_tokens: 2559
- cache_read_tokens: 39814
- cache_write_tokens: 13162
- cost_usd: 0.165287
