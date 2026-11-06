# Cross-Cycle Insights (through cycle 0007)

## Recurring wins

- Changes to the admission contest are what keep paying off: who competes (admission window, cycle 0001), which resident is the victim (SLRU probation in cycles 0001–0003, coldest-of-4 sampled probation victim in cycle 0005), and, on early lineages only, how fresh the evidence is.
- Structural changes beat timing changes. The window, SLRU, and the sampled victim each added on top of the previous best. Changes to periods, sample rates and reset intervals mostly did not.
- Adaptivity helped once, at the coarse level: the hill-climbing window (cycle 0002) fixed the undersized fixed window at small sizes. Every later refinement of it failed.
- Changes that make the cache better at picking by frequency (a sharper contest bar, less noisy estimates) reliably help web_0, especially at the small size.
- The best base is still cycle 0005 v003: adaptive window, adaptive protected split, coldest-of-4 sampled probation victim, double counting kept, strict `>`. Cycles 0006 and 0007 named cohort winners, but neither improved on this base.

## Recurring failures

- Feedback-loop and controller redesign has failed every time it was tried after the first adaptive window: fixed sample periods, sample periods tied to turnover, alternating dual climbers, and ARC-style ghost controllers. The causes were lagging or noisy signals, integer step truncation, warmup bias, and ghost lists filling unevenly because the frequency filter, not the window size, rejects most candidates.
- Anything with a horizon or period that scales with capacity misbehaves at one end of the size range (aging periods, climber samples, capacity/2 ghost horizons).
- One-knob changes to the sketch or the tie-break upset the admission bar. Aging alone, conservative update alone, single counting alone, `>=` ties, a wider victim sample, and re-entry bonuses on their own each made it either too easy or too hard to clear, usually costing web_0 at the large size.
- Stacking frequency-favouring mechanisms (single count, a wider minimum, the sampled victim) overlaps instead of adding, so returns shrink or turn negative.
- Hypotheses aimed at hm_0 (especially at the small size) have missed their stated mechanism in almost every cycle.
- Pure latency or metadata changes cannot win, because the fitness vector scores hit-rate only. Cost has been flagged as unmeasured for seven cycles.

## Open questions

- Paired sketch changes that hold the admission bar in place. Does aging plus single counting work? Does conservative update plus a tie-break toward the candidate (perhaps above a small floor) keep the hm_0 small gain and recover web_0?
- Is the right model to treat the sketch, the double count and the `>` rule as one coupled admission policy, and to tune them jointly?
- Does hm_0 small want fewer false admissions (stickier incumbents, which showed a hint in cycle 0007) and not more recency? The second reading would explain why recency or re-entry tweaks keep missing.
- Could a less noisy victim estimate, or a contest margin, sharpen admission better than widening the sample?
- If a controller is ever revisited, could a balanced, batched counterfactual signal (normalised ghosts, small steps, suppressed until the cache is full) work?
- Can per-op-time and metadata-size be added to the evaluator so that cost-reducing work becomes rewardable?

## Per-policy notes

- **Admission window:** Essential, since it fixes the reject-every-first-timer flaw. Adaptive sizing helps at small sizes. The climber's timing is the persistent weak spot, so keep it frozen.
- **SLRU main region (80/20 protected/probation):** Reliable gain on web_0, about flat or slightly negative on hm_0. An adaptive protected split gave only a marginal, noisy win, and 80/20 may already be near optimal.
- **Victim selection:** Coldest-of-4 from the LRU end of probation is the sweet spot. Widening to 8 picks entries that only look cold. Further victim-choice tweaks at the LRU end are exhausted.
- **Count-min sketch:** In this lineage it acts as the admission policy, not a measurement tool. Low-count ties and near-ties decide outcomes, and collision inflation rarely does. The double count on miss-then-insert is the newcomers' only recency weight. It is a known correctness defect, but removing it alone hurts. Keep independent row hashing, since it carries real accuracy on web_0 large.
- **Aging/decay:** The verdict has flipped between cycles. It helped before SLRU (cycle 0002), hurt on top of the adaptive window (cycle 0003), and alone amplified newcomer bias (cycle 0007). It only seems viable when paired with single counting.
- **Admission tie-break:** Strict `>` (victim wins ties) has beaten `>=` or recency ties on the current lineage. That may need revisiting only if the sketch's tie profile changes (for example, with conservative update).
- **Ghost lists / re-entry bonuses:** Poor fit for W-TinyLFU so far. If tried again, use only a fixed, short horizon.
- **Traces:** web_0 at the large size is the lineage's most valuable and most fragile cell, and almost any change to the admission bar hurts it. hm_0 is the chronic hard trace, especially at the small size, where a plain LRU already captures much of the reuse.
