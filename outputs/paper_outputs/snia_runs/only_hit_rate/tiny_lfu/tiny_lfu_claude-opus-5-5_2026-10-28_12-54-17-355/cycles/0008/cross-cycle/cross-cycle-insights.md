## Recurring wins

- Structural changes to the admission/eviction contest are where gains have come from: the admission window (c1), the adaptive window (c2), the SLRU main region (c3), and the sampled probation victim, coldest of 4 (c5). Each one changed who competes or who the victim is.
- Changes that make the cache better at choosing by frequency keep helping web_0. Changes that try to rescue recency-driven entries keep missing.
- Cap-free counters are better than the 15-cap. The cap caused ties at the top of the count range and a regression on web_0 large (c1–c2).
- Keeping strict `>` admission, where the victim wins ties, has been the safer rule in every sketch variant tried (c4, c7, c8).
- Cycle 0005 v003 (adaptive window + adaptive protected split + coldest-of-4 victim, double counting kept) is the real best base. Cycles 6–8 produced only cohort-relative winners that did not beat it.

## Recurring failures

- Window and protected-split controller redesigns have failed in every attempt (c3–c5): fixed or turnover-tied sample periods, alternating-turn climbers, steps lost to integer truncation, ARC-style ghost controllers. The causes are noisy or lagging feedback and lazy rebalancing.
- Any horizon or period that scales with capacity drifts at one end of the size range. This applies to aging periods, climber samples, ghost sizes and the sticky reuse flag.
- Tie-breaks in the candidate's favour have hurt the large cells every time: global, recency-based, floored, and reuse-gated (c4, c7, c8). Equal frequency is not a good enough reason to evict a main resident.
- One-knob sketch changes upset a finely balanced admission bar. Halving, conservative update, single counting, double hashing and wider victim samples each hurt web_0, most of all web_0 large (c3, c6, c7).
- Pairing changes so they cancel out has not worked so far. Integer halving with single counting moved the asymmetry instead of removing it (c8).
- Stacking frequency-favouring mechanisms gives diminishing or negative returns. A wider minimum over noisy estimates picks residents that only look cold.
- Pure latency work cannot win, because the fitness vector holds only hit-rate. Metadata-size and per-op-time have stayed unmeasured for all 8 cycles.

## Open questions

- Would aging work if rounding favours the resident (count 0 treated as 1 in the contest, or a floored contest), paired with single counting?
- Would changing where admissions land in probation, or how long they stay there, finally move hm_0 small? Admission-contest tweaks have not.
- Can a reuse signal normalised to residency time or window size keep small-size gains without the large-size drift? This is the last tie-related idea worth one try.
- Does conservative update's loss on web_0 come from the estimate-2 tie on its own, or from the broader reshaping of counts?
- Can per-op-time and metadata-size be added to the evaluator, so that cost is scored and latency work can be rewarded?

## Per-policy notes

- W-TinyLFU lineage: in this lineage the sketch, the double count and the `>` rule work as one coupled admission policy, not a neutral frequency estimate. The double count's estimate of 2 on miss-then-insert is newcomers' only recency weight in an unaged sketch. Most admission decisions are made by ties and near-ties at low counts, not by collision inflation.
- Window: adaptive beats a fixed 1% window, mainly at small sizes. Controller timing tweaks have been settled as harmful, so freeze the controller unless a balanced, batched signal is proposed.
- SLRU / probation: the 80/20 split, or the adaptive split, is close to a local optimum. Victim choice at the LRU end is exhausted; a sample of 4 is the sweet spot.
- Trace regularities: web_0 large is the most valuable and most fragile cell, and changes that lower the admission bar break it first. hm_0, especially small, has stayed flat or regressed across 8 cycles of admission tweaks. Stickier incumbents helped it slightly (c7–c8), which suggests it wants fewer false admissions, not more recency. The typical cohort shape is gains at small sizes and losses at large sizes.
