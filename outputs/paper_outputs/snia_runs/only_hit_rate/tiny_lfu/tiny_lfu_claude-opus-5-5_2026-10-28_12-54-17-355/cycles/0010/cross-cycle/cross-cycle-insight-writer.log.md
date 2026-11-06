# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-28T14:07:55+00:00

## timeline

### 14:08:14 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0010)\n\n## Recurring wins\n\n- Structural changes to the admission/eviction contest keep paying off. Examples: the admission window (0001), the adaptive window (0002), the SLRU main region (0003) and coldest-of-4 probation victim sampling (0005). Each one changed *who competes* or *who is the victim*. Changes to timing and feedback loops almost never pay off.\n- Making frequency-based selection sharper helps, especially on web_0. The large-cell frequency filter is the lineage's most valuable and most fragile asset.\n- Recency evidence works only when it is normalised to the entry's own residency. The residency-normalised late-hit tie gate (0009) is the only tie-related change that did not hurt web_0 large. Signals scaled to the window or to capacity drifted at large sizes (0008).\n- Rounding in the resident's favour, such as ceiling halving, closes the reopened admission bar that integer halving creates (0008→0009).\n- The adaptive protected split does real work on hm_0 large (0004, 0010). It is the only structural knob shown to matter for hm_0.\n- Keeping strict `>` admission, where the victim wins ties, has been confirmed repeatedly as better than tie-breaks that favour the candidate.\n\n## Recurring failures\n\n- Loosening the admission bar trades small-cell gains for large-cell losses, worst on web_0 large. This happened in 0004, 0006, 0007, 0008 and 0010. Tie-breaks have failed in every form: global, floored, reuse-gated and loosened. If the tie gate moves at all, it should tighten.\n- Window/climber controller redesign has failed for about six cycles in a row. Constant sample periods, periods scaled to capacity or to turnover, alternating turns, ghost-list (ARC-style) controllers and extra climber turns have all failed. The causes were lazy rebalancing lag, noisy deltas, integer step truncation, and step decay per climb.\n- Every horizon scaled to capacity misbehaves at one end of the size range. This covers aging periods, climber sample periods, ghost sizes and sticky window flags. A fixed period only moves the failure to the other end (0010).\n- Aging combined with double counting does not work, whatever the period or rounding. Newcomers keep the full double increment while residents decay. Aging on its own does not add recency (0007).\n- One-knob sketch changes fail: conservative update, halving alone, cheaper or correlated hashing, wider victim samples. Together, the sketch's error profile, double counting and the `>` rule *are* the admission policy, and they are finely balanced.\n- Placement changes that put high-count entries at the probation head (0009) fail. The head sample sets the admission bar, so placement and admission are coupled.\n- Several cycles were \"won\" only relative to the cohort (0006, 0007, 0008, 0010), not against the actual best base. Track the true base explicitly.\n- Per-op-time and metadata-size have never been measured in ten cycles. Latency-only changes cannot win, and the costs of metadata and extra estimates are invisible.\n\n## Open questions\n\n- Does *tightening* the late-hit tie gate, or removing the tie exception at large window sizes, strengthen the large cells further?\n- Can the protected-split climber's step, floor or bounds be tuned as a deliberate lever for hm_0 large?\n- Would retuning the window climber's step decay or sample period, measured against rebalance completion, help the large cells? Are the climbers better left frozen?\n- Is aging viable only when paired with single counting on a fixed, capacity-independent period, so the two effects can be separated?\n- What lever closes the web_0 small gap without using the tie rule?\n- hm_0 small and web_0 small look traded against each other through the strength of the frequency contest. Is there any change that lifts hm_0 small without draining web_0 small?\n- Can the evaluator add per-op-time and metadata-size so that the cost of each design becomes visible and can be rewarded?\n\n## Per-policy notes\n\n- **Current base:** adaptive window + adaptive protected split + 80/20-seeded SLRU + coldest-of-4 sampled probation victim + double-count sketch with no aging + strict `>` with a residency-normalised late-hit tie exception (cycle 0009 v001).\n- **web_0 (scan-like, one-hit heavy):** Gains come from SLRU, victim sampling and a sharper frequency contest. The large cell breaks whenever the bar drops. It is the main regression signal for every change.\n- **hm_0 (stable, recency-heavy):** This is the hard trace. It is mostly flat or hurt through ten cycles of admission tweaks. hm_0 small leans slightly toward stickier incumbents and fewer false admissions, not more recency. hm_0 large responds to the protected-split boundary. Any gain on hm_0 small that costs web_0 small counts as no progress.\n- **Size axis:** Small cells reward a lower admission bar and larger windows. Large cells punish both. Treat any small-only gain with suspicion until the large cells are shown to hold.\n"
}
```

### 14:08:19 chat

I've written `cross-cycle-insights.md` with the four required sections, built only from cycles 0001–0010 in the ledger.

- **Best base so far:** cycle 0009 v001. It is TinyLFU with an adaptive window, an adaptive protected split, a sampled probation victim, double counting with no aging, and strict `>` admission with one exception for "late window hits" measured against residency. Four cycles (0006, 0007, 0008, 0010) named a winner only relative to their cohort, so the file tracks the true base explicitly.
- **What keeps working:** changes to who competes for admission or who gets evicted. Recency evidence helps only when it is measured against how long the entry has been resident.
- **What keeps failing:**
  - Anything that lowers the admission bar. It helps small caches and hurts large ones, worst on web_0 at the large size.
  - Redesigns of the window/climber controller.
  - Timing periods that scale with cache capacity.
  - Aging while double counting is kept.
  - Changing one part of the sketch on its own.
- **Main open threads:**
  - A tighter tie gate.
  - The protected-split climber as a lever for hm_0 at the large size.
  - Whether hm_0 small can be lifted without draining web_0 small.
- **Still unmeasured:** per-op-time and metadata-size have not been measured in any of the ten cycles.

The file also has per-policy notes for each trace (web_0, hm_0) and for small versus large cache sizes.


---

- finished: 2026-10-28T14:08:20+00:00
- duration: 24.45s
- status: ok
- input_tokens: 4
- output_tokens: 2340
- cache_read_tokens: 44653
- cache_write_tokens: 16568
- cost_usd: 0.188291
