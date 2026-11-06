# Cross-Cycle Insights (through cycle 0010)

## Recurring wins

- Structural changes to the admission/eviction contest keep paying off. Examples: the admission window (0001), the adaptive window (0002), the SLRU main region (0003) and coldest-of-4 probation victim sampling (0005). Each one changed *who competes* or *who is the victim*. Changes to timing and feedback loops almost never pay off.
- Making frequency-based selection sharper helps, especially on web_0. The large-cell frequency filter is the lineage's most valuable and most fragile asset.
- Recency evidence works only when it is normalised to the entry's own residency. The residency-normalised late-hit tie gate (0009) is the only tie-related change that did not hurt web_0 large. Signals scaled to the window or to capacity drifted at large sizes (0008).
- Rounding in the resident's favour, such as ceiling halving, closes the reopened admission bar that integer halving creates (0008→0009).
- The adaptive protected split does real work on hm_0 large (0004, 0010). It is the only structural knob shown to matter for hm_0.
- Keeping strict `>` admission, where the victim wins ties, has been confirmed repeatedly as better than tie-breaks that favour the candidate.

## Recurring failures

- Loosening the admission bar trades small-cell gains for large-cell losses, worst on web_0 large. This happened in 0004, 0006, 0007, 0008 and 0010. Tie-breaks have failed in every form: global, floored, reuse-gated and loosened. If the tie gate moves at all, it should tighten.
- Window/climber controller redesign has failed for about six cycles in a row. Constant sample periods, periods scaled to capacity or to turnover, alternating turns, ghost-list (ARC-style) controllers and extra climber turns have all failed. The causes were lazy rebalancing lag, noisy deltas, integer step truncation, and step decay per climb.
- Every horizon scaled to capacity misbehaves at one end of the size range. This covers aging periods, climber sample periods, ghost sizes and sticky window flags. A fixed period only moves the failure to the other end (0010).
- Aging combined with double counting does not work, whatever the period or rounding. Newcomers keep the full double increment while residents decay. Aging on its own does not add recency (0007).
- One-knob sketch changes fail: conservative update, halving alone, cheaper or correlated hashing, wider victim samples. Together, the sketch's error profile, double counting and the `>` rule *are* the admission policy, and they are finely balanced.
- Placement changes that put high-count entries at the probation head (0009) fail. The head sample sets the admission bar, so placement and admission are coupled.
- Several cycles were "won" only relative to the cohort (0006, 0007, 0008, 0010), not against the actual best base. Track the true base explicitly.
- Per-op-time and metadata-size have never been measured in ten cycles. Latency-only changes cannot win, and the costs of metadata and extra estimates are invisible.

## Open questions

- Does *tightening* the late-hit tie gate, or removing the tie exception at large window sizes, strengthen the large cells further?
- Can the protected-split climber's step, floor or bounds be tuned as a deliberate lever for hm_0 large?
- Would retuning the window climber's step decay or sample period, measured against rebalance completion, help the large cells? Are the climbers better left frozen?
- Is aging viable only when paired with single counting on a fixed, capacity-independent period, so the two effects can be separated?
- What lever closes the web_0 small gap without using the tie rule?
- hm_0 small and web_0 small look traded against each other through the strength of the frequency contest. Is there any change that lifts hm_0 small without draining web_0 small?
- Can the evaluator add per-op-time and metadata-size so that the cost of each design becomes visible and can be rewarded?

## Per-policy notes

- **Current base:** adaptive window + adaptive protected split + 80/20-seeded SLRU + coldest-of-4 sampled probation victim + double-count sketch with no aging + strict `>` with a residency-normalised late-hit tie exception (cycle 0009 v001).
- **web_0 (scan-like, one-hit heavy):** Gains come from SLRU, victim sampling and a sharper frequency contest. The large cell breaks whenever the bar drops. It is the main regression signal for every change.
- **hm_0 (stable, recency-heavy):** This is the hard trace. It is mostly flat or hurt through ten cycles of admission tweaks. hm_0 small leans slightly toward stickier incumbents and fewer false admissions, not more recency. hm_0 large responds to the protected-split boundary. Any gain on hm_0 small that costs web_0 small counts as no progress.
- **Size axis:** Small cells reward a lower admission bar and larger windows. Large cells punish both. Treat any small-only gain with suspicion until the large cells are shown to hold.
