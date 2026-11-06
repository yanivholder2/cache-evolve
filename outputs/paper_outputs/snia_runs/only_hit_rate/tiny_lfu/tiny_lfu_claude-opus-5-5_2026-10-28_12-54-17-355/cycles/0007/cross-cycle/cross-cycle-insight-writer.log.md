# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-28T13:44:35+00:00

## timeline

### 13:44:55 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0007)\n\n## Recurring wins\n\n- Changes to the admission contest are what keep paying off: who competes (admission window, cycle 0001), which resident is the victim (SLRU probation in cycles 0001–0003, coldest-of-4 sampled probation victim in cycle 0005), and, on early lineages only, how fresh the evidence is.\n- Structural changes beat timing changes. The window, SLRU, and the sampled victim each added on top of the previous best. Changes to periods, sample rates and reset intervals mostly did not.\n- Adaptivity helped once, at the coarse level: the hill-climbing window (cycle 0002) fixed the undersized fixed window at small sizes. Every later refinement of it failed.\n- Changes that make the cache better at picking by frequency (a sharper contest bar, less noisy estimates) reliably help web_0, especially at the small size.\n- The best base is still cycle 0005 v003: adaptive window, adaptive protected split, coldest-of-4 sampled probation victim, double counting kept, strict `>`. Cycles 0006 and 0007 named cohort winners, but neither improved on this base.\n\n## Recurring failures\n\n- Feedback-loop and controller redesign failed in five cycles straight (0003–0007 touched it, or chose to freeze it): fixed sample periods, sample periods tied to turnover, alternating dual climbers, and ARC-style ghost controllers. The causes were lagging or noisy signals, integer step truncation, warmup bias, and ghost lists filling unevenly because the frequency filter, not the window size, rejects most candidates.\n- Anything with a horizon or period that scales with capacity misbehaves at one end of the size range (aging periods, climber samples, capacity/2 ghost horizons).\n- One-knob changes to the sketch or the tie-break upset the admission bar. Aging alone, conservative update alone, single counting alone, `>=` ties, a wider victim sample, and re-entry bonuses on their own each made it either too easy or too hard to clear, usually costing web_0 at the large size.\n- Stacking frequency-favouring mechanisms (single count, a wider minimum, the sampled victim) overlaps instead of adding, so returns shrink or turn negative.\n- Hypotheses aimed at hm_0 (especially at the small size) have missed their stated mechanism in almost every cycle.\n- Pure latency or metadata changes cannot win, because the fitness vector scores hit-rate only. Cost has been flagged as unmeasured for seven cycles.\n\n## Open questions\n\n- Paired sketch changes that hold the admission bar in place. Does aging plus single counting work? Does conservative update plus a tie-break toward the candidate (perhaps above a small floor) keep the hm_0 small gain and recover web_0?\n- Is the right model to treat the sketch, the double count and the `>` rule as one coupled admission policy, and to tune them jointly?\n- Does hm_0 small want fewer false admissions (stickier incumbents, which showed a hint in cycle 0007) and not more recency? The second reading would explain why recency or re-entry tweaks keep missing.\n- Could a less noisy victim estimate, or a contest margin, sharpen admission better than widening the sample?\n- If a controller is ever revisited, could a balanced, batched counterfactual signal (normalised ghosts, small steps, suppressed until the cache is full) work?\n- Can per-op-time and metadata-size be added to the evaluator so that cost-reducing work becomes rewardable?\n\n## Per-policy notes\n\n- **Admission window:** Essential, since it fixes the reject-every-first-timer flaw. Adaptive sizing helps at small sizes. The climber's timing is the persistent weak spot, so keep it frozen.\n- **SLRU main region (80/20 protected/probation):** Reliable gain on web_0, about flat or slightly negative on hm_0. An adaptive protected split gave only a marginal, noisy win, and 80/20 may already be near optimal.\n- **Victim selection:** Coldest-of-4 from the LRU end of probation is the sweet spot. Widening to 8 picks entries that only look cold. Further victim-choice tweaks at the LRU end are exhausted.\n- **Count-min sketch:** In this lineage it acts as the admission policy, not a measurement tool. Low-count ties and near-ties decide outcomes, and collision inflation rarely does. The double count on miss-then-insert is the newcomers' only recency weight. It is a known correctness defect, but removing it alone hurts. Keep independent row hashing, since it carries real accuracy on web_0 large.\n- **Aging/decay:** The verdict has flipped between cycles. It helped before SLRU (cycle 0002), hurt on top of the adaptive window (cycle 0003), and alone amplified newcomer bias (cycle 0007). It only seems viable when paired with single counting.\n- **Admission tie-break:** Strict `>` (victim wins ties) has beaten `>=` or recency ties on the current lineage. That may need revisiting only if the sketch's tie profile changes (for example, with conservative update).\n- **Ghost lists / re-entry bonuses:** Poor fit for W-TinyLFU so far. If tried again, use only a fixed, short horizon.\n- **Traces:** web_0 at the large size is the lineage's most valuable and most fragile cell, and almost any change to the admission bar hurts it. hm_0 is the chronic hard trace, especially at the small size, where a plain LRU already captures much of the reuse.\n"
}
```

### 13:44:57 thinking



### 13:44:58 tool: Edit

```json
{
  "replace_all": false,
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "old_string": "failed in five cycles straight (0003–0007 touched it, or chose to freeze it): fixed",
  "new_string": "has failed every time it was tried after the first adaptive window: fixed"
}
```

### 13:45:04 chat

I wrote `cross-cycle-insights.md` from the ledger for cycles 0001–0007, using the four required sections in order.

- **Best base:** The best version is still cycle 0005 v003. That's the adaptive window, the 80/20 protected/probation split, picking the coldest of the 4 oldest probation entries for eviction, double counting kept, and strict `>` admission. Cycles 0006 and 0007 each named a winner within their cohort, but neither one beat this base.
- **Recurring wins:** Gains keep coming from changes to the admission contest, such as the window, the SLRU split and the sampled victim. Changes that alter structure have added up; changes that alter timing have not.
- **Recurring failures:**
  - Every attempt to redesign the window or split controllers after the first adaptive window failed.
  - Anything whose period or horizon grows with cache size misbehaved at one end of the size range.
  - Changing one part of the sketch or the tie-break on its own (for example aging alone, or conservative update alone) moved the admission bar, usually costing web_0 at the large size.
  - Ideas aimed at hm_0 at the small size kept missing.
  - Speed and memory costs have gone unmeasured for seven cycles, so changes that only make things faster can't score.
- **Open questions:** Whether paired sketch changes work: aging together with single counting, and conservative update together with a tie-break toward the newcomer. Whether hm_0 at the small size needs fewer wrong admissions rather than more recency. Whether speed and memory can be added to the evaluator.

The per-policy notes also record that the verdict on sketch aging has reversed several times across cycles.


---

- finished: 2026-10-28T13:45:04+00:00
- duration: 28.88s
- status: ok
- input_tokens: 6
- output_tokens: 2864
- cache_read_tokens: 70123
- cache_write_tokens: 14013
- cost_usd: 0.183433
