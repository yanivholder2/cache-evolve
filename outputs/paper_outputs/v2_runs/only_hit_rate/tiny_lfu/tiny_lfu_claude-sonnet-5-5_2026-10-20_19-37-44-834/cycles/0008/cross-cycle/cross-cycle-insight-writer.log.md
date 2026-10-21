# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:11:15+00:00

## timeline

### 20:11:24 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that preserve or improve frequency-estimate quality are neutral-to-slightly-positive: wider sketch rows (cycles 7, 8), the tie-admit duel (cycle 3), and behavior-preserving rewrites (single-hash indexing, index reuse, slab links).\n- Width moves hit-rate monotonically with sharply diminishing returns. Wider helps slightly and narrower hurts.\n- Count semantics and the admit threshold are coupled. Retuning them together restored parity (cycle 8).\n\n## Recurring failures\n\n- Changes that discard or degrade frequency information lose hit-rate. Examples: aging or halving at any period, doorkeeper, narrower rows, 4-bit saturating counters with a shared hash, blocked correlated rows, and admission looser than the tie-admit duel.\n- Segment-internal tuning is flat: window size (including adaptive), protected share, demotion choice, and probation victim sampling. Workload-a appears saturated and stable, so these only produce noise.\n- Winners are repeatedly ties or least-bad variants chosen on hit-rate among near-equals, not real gains.\n- Evaluation has been blind for eight cycles. Fitness reports hit-rate only, with no metadata-size or per-op-time, and only one trace (workload-a) at one cache size. The cost-oriented ideas have never been judged on their actual goal.\n- Variant authors cannot read overlay source, so explanations are unconfirmed. Some hypothesis text has muddled the direction of a width factor.\n\n## Open questions\n\n- Do the cost candidates (single-hash, index reuse, slab links, single-count with a lower threshold, width 8 vs 16) actually save size or time? This needs metadata-size and per-op-time in the fitness output.\n- Would a second trace (shifting popularity, a different cache size) reveal value in aging, a doorkeeper, or adaptive windows that workload-a hides?\n- Is workload-a saturated for hit-rate, so that only cost work remains?\n- Can blocked or compact sketches work with decorrelated per-row hashes?\n- What is the best width/size trade-off point?\n\n## Per-policy notes\n\n- TinyLFU, the only policy so far, is a window plus segmented main region, a sketch-gated admission duel, and a sketch with conservative update. A clean base is the tie-admit duel with single-count and a lower threshold on the baseline-or-wider sketch.\n- Drop: aging and halving, doorkeeper, window tuning, protected sizing, demotion and probation victim tweaks, width cuts, and further admission loosening.\n- Keep as unverified cost candidates: single-hash indexing, index reuse, slab links, and intermediate width.\n- Any width hypothesis must state explicitly which direction its factor scales the sketch.\n"
}
```

### 20:11:28 chat

I wrote `cross-cycle-insights.md` in the cycle 0008 cross-cycle output dir. It has the four fixed sections in order: Recurring wins, Recurring failures, Open questions and Per-policy notes. I regenerated it from the ledger digest alone and did not read the previous version.

The main signals it records across the eight cycles:

- **Wins:** Changes that keep frequency information intact are neutral or slightly positive. Wider rows help a little, with sharply diminishing returns, and count semantics and the admit threshold have to be tuned together.
- **Failures:** Anything that discards or degrades frequency information loses hit-rate, including aging, a doorkeeper, narrower rows and looser admission. Segment-internal tweaks are flat. Winners are mostly ties, and the fitness output has reported hit-rate only for all eight cycles.
- **Open questions:** The cost-oriented ideas have never been measured on size or speed. A second trace is needed, and workload-a may already be saturated.

I kept the notes qualitative and added no numbers.


---

- finished: 2026-10-20T20:11:28+00:00
- duration: 13.13s
- status: ok
- input_tokens: 4
- output_tokens: 1428
- cache_read_tokens: 39575
- cache_write_tokens: 11199
- cost_usd: 0.063771
