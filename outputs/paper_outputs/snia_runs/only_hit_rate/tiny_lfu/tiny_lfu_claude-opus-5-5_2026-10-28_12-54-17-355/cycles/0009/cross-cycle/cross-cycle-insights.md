# Cross-Cycle Insights (through cycle 0009)

## Recurring wins

- Structural changes to the admission contest are the steady source of progress. What has worked: who competes (the window, c1), which entry is the victim (SLRU c3, coldest-of-4 probation sample c5), and how big the window is (adaptive window c2). The parts of W-TinyLFU that make frequency-based choices sharper keep paying off.
- Cycle 0005 v003 (adaptive window + adaptive protected split + coldest-of-4 sampled victim + double counting + strict `>`) is the last clear, regression-free gain. Cycles 6–8 produced only cohort-relative winners that did not beat it.
- Normalising recency evidence to residency or scale works where raw signals drift. The residency-normalised late-hit tie (c9) is the first tie-related change that held web_0 large. Removing the counter cap (c2) and rounding toward the resident in aging (c9) both fixed the specific asymmetries they targeted.
- Clear wins have come from one-axis changes evaluated against a stable base. Stacking mechanisms that point the same way ("favour accumulated frequency") has given diminishing or negative returns (c6).

## Recurring failures

- **Window/protected climber redesigns:** 0 for 5 cycles (c3–c5 and related). These include fixed or turnover-tied sample periods, alternating two-knob climbers, and ARC-style ghost controllers. Each failed for one of these reasons: noisy deltas, lazy-rebalance lag, integer-truncated steps, warmup bias, or unbalanced ghost inputs.
- **Horizons that scale with capacity:** these keep breaking at one end of the size range. Aging periods, climber sample periods, ghost sizes, the sticky reuse flag in large windows, and the c9 ceiling-halving period all fire too often at small sizes or too rarely at large ones.
- **Candidate-favouring tie-breaks** hurt the large cells, web_0 large most of all. This has held under three different sketches: global `>=` (c4), floored `>=` with conservative update (c8), and the sticky reuse gate (c8). Equal frequency is not a good enough reason to evict a probation resident.
- **One-knob sketch changes** upset the admission bar. Single counting, halving, conservative update, a wider victim minimum and correlated double hashing all moved it in ways nobody intended (c5–c8). The sketch, the double count and strict `>` act as one coupled admission policy.
- **Placement tweaks that put high-count entries at the probation head** raise the sampled-victim bar and evict entries with real reuse. Demote-to-head regressed in all cells (c9).
- **Victim-choice tweaks at the LRU end** (sample size beyond 4) do not help. Taking the minimum over more noisy estimates picks residents that only look cold.
- **Per-op-time and metadata-size have never been measured** in 9 cycles. Latency-only changes therefore can only lose (c7), and no metadata cost has ever been weighed.

## Open questions

- Does a looser or window-scaled late-hit cutoff keep c9 v001's large-cell recovery and restore web_0 small? Does that lineage beat c5 v003 outright?
- Does aging on a fixed period that does not scale with capacity, with double counting kept, help on its own? This would separate the effect of halving from the effect of single counting.
- Can hm_0 small improve without draining web_0 small? The two now look directly traded against each other through how strong the frequency contest is. Candidate levers are a longer probation stay for new admissions that does not change what the head sample sees, or fewer false admissions (sticky incumbents helped slightly in c7).
- Can the evaluator add per-op-time and metadata-size, so cost-side work becomes rewardable?

## Per-policy notes

- **TinyLFU lineage (only policy explored):** baseline → 1% window (c1) → adaptive window (c2) → +80/20 SLRU (c3) → +adaptive protected split (c4, marginal) → +coldest-of-4 probation victim (c5, best clear base) → residency-normalised late-hit tie (c9, promising, outright gain unconfirmed).
- **web_0:** has scan-like one-hit traffic. It gains from SLRU, victim sampling and a sharp frequency filter. web_0 large is the lineage's most valuable and most fragile cell. Almost every change that lowers the admission bar hurts it.
- **hm_0:** a plain LRU already holds much of its reuse set, so gains there are small. hm_0 small has been the hard cell for every cycle, and every hypothesis aimed at it has had its predicted mechanism refuted. It responds slightly to stickier incumbents, and it relied by accident on the double-count bonus for re-entering keys.
- **Size shape:** cohort deltas typically show small-size gains alongside large-size losses. Judge a change by the large cells first.
