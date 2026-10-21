# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:30:30+00:00

## timeline

### 04:30:41 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Structural separation of one-touch from re-referenced entries (a probationary/protected split inside each bucket) is the one lever that clearly paid off. It beat both geometry and promotion tweaks.\n- A small probation segment (about slots/16, roughly 4 slots) won from several independent routes. The size trend has turned, so this is at or near the optimum.\n- A lean positional split with no per-entry state (no ghost, counters or adaptive targets) captures almost all the gain over plain LRU. It was rebuilt on the baseline more than once.\n- Wider associativity stacks with the small-probation split. This is only shown against the baseline, not as an increment over the best parent.\n- First-hit promotion with full move-to-back recency is hard to beat.\n\n## Recurring failures\n\n- Weakening or delaying promotion (transposition, midpoint, half-promotion, two-hit rule) lost hit rate. Short buckets and small probation leave no room to wait for a second hit.\n- Ghost history and ghost-hit promotion added nothing. A ghost hit is weak reuse evidence here and can displace entries that had earned protection.\n- Small-window frequency victim selection and recency-stamp overrides of probation-first eviction were flat or negative. The segment boundary already encodes the reuse signal.\n- Placement outside the hashed bucket (two-choice) fails the slot-legality invariant, even on the baseline.\n- Probation-size shrinking below the sweet spot, and demotion-placement tweaks, are flat or slightly worse. Both are exhausted.\n- Per-op-time and metadata-size have never been measured, so no cost-side claim has been confirmed or refuted. Several \"wins\" are exact ties, and some are comparisons to the baseline rather than the parent.\n\n## Open questions\n\n- Can per-op-time and metadata-size be added to the fitness vector? This blocks judging the rotate rewrite, wider buckets and the \"lean\" claims.\n- Does the wider-associativity gain hold as an increment on the current best parent once scan and shuffle cost is counted?\n- Are the exact-tie variants truly neutral, or are they edits that never changed behaviour, such as a demotion path that rarely fires?\n- Do the results hold on traces other than the LIRS-style one, or is the tuning specific to it?\n- Is there any headroom left in protected-segment handling, or is the design at its ceiling for this trace?\n- Could a stricter ghost rule (a ghost hit plus a second signal) rescue ghost history? This is low priority.\n\n## Per-policy notes\n\n- Plain move-to-back LRU: the baseline. Every policy tweak that weakened promotion lost to it. Wider buckets gave only a small gain, so hash imbalance is a minor miss source.\n- Segmented LRU (probation + protected): the winning family, and policy rather than geometry limits hit rate. Lean positional version with a small fixed probation is the reference base. Further levers inside it (demotion placement, promotion rules, size, stamps) are exhausted.\n- Ghost-ring and adaptive 2Q-style variants: no gain over the fixed split, and they add unpriced state. Do not pursue without a stricter promotion rule.\n- Frequency-aware and two-choice variants: frequency windows were neutral, and two-choice was rejected at the invariant gate.\n"
}
```

### 04:30:45 chat

I wrote `cross-cycle-insights.md` to the cycle 0006 cross-cycle output dir, using the four fixed sections in order. I didn't re-read it after writing.

- **Recurring wins:** The probationary/protected split inside each bucket is the one lever that clearly paid off. A small probation segment (about 4 slots, roughly slots/16) is at or near the optimum. The lean version with no extra per-entry state captures almost all the gain. Wider associativity stacks on top, but that is only shown against the baseline.
- **Recurring failures:**
  - Weakening or delaying promotion lost hit rate.
  - Ghost history added nothing.
  - Frequency windows and recency-stamp overrides were flat or negative.
  - Placement outside the hashed bucket fails the invariant gate.
  - Per-op-time and metadata-size have never been measured, so no cost-side claim has been confirmed.
- **Open questions:** The main ones are getting per-op-time and metadata-size into the fitness vector, and whether the associativity gain holds as an increment on the current best parent. The rest are whether the exact-tie variants are truly neutral, whether results hold on traces other than the LIRS-style one, and whether any headroom is left in protected-segment handling.
- **Per-policy notes:** Grouped by plain LRU, segmented LRU, ghost-ring and adaptive variants, and frequency-aware and two-choice variants.

I used only qualitative terms and no numbers beyond the probation size the ledger itself states.


---

- finished: 2026-10-21T04:30:46+00:00
- duration: 15.52s
- status: ok
- input_tokens: 4
- output_tokens: 1750
- cache_read_tokens: 37883
- cache_write_tokens: 9671
- cost_usd: 0.060541
