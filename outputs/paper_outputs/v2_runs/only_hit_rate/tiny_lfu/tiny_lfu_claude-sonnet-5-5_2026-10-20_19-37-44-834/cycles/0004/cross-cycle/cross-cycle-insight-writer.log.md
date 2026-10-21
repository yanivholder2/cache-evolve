# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:56:12+00:00

## timeline

### 19:56:24 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that preserve frequency information and only loosen admission slightly (cycle 3 tie-admit duel) give tiny gains; this is the only kind of change that has helped.\n- Changes that leave frequency estimates intact (segment split, single-hash row derivation) are neutral on hit-rate, so they are safe candidates for cost wins.\n- Winners are repeatedly ties or marginal gains near noise (cycles 1-4), not clear improvements.\n\n## Recurring failures\n\n- Anything that discards or hides frequency information lost hit-rate: aging/decay, larger window, doorkeeper, 4-bit saturating counters, narrower sketch rows.\n- Hit-rate-only tuning of window size, protected-segment share and admission on workload-a returns roughly nothing. The workload looks stable, frequency-driven and saturated.\n- Adaptive window tuning has no way to show value on a single stable trace.\n- Evaluation gap: for four cycles the fitness vectors have reported hit-rate only. Metadata-size and per-op-time, the stated goals of the cost-oriented ideas, remain unmeasured, so no cost trade has been judged.\n- Environmental failures (cycle 1 download rate limit) can block an idea without testing it. Variant authors often cannot read overlay source, so explanations are plausible and not confirmed.\n\n## Open questions\n\n- Fix the evaluation first. Add metadata-size and per-op-time to the fitness output, plus a second trace (shifting popularity or a different cache size).\n- Does aging pay off on a trace with real popularity shifts or a longer sample period?\n- Does a doorkeeper help with the admission +1 offset adjusted, or under real sketch pollution?\n- Does single-hash row derivation actually cut per-op-time?\n- Would a gentler width reduction than 2x keep hit-rate while saving size?\n- Can a compact 4-bit sketch recover with aging and decorrelated row indices, and do its savings justify any loss?\n- Are the small wins (tie-admit duel) real or noise?\n\n## Per-policy notes\n\n- TinyLFU, windowed with segmented main (current base includes the tie-admit duel): the baseline's sketch-gated admission already captures most of the benefit on workload-a.\n- Sketch: u8 counters with conservative update lost no accuracy. 4-bit counters with a shared hash lost hit-rate, and halving the row width lost hit-rate. Single 64-bit hash indexing held hit-rate level and is a candidate cost win to verify on speed.\n- Window: both larger and adaptively tuned windows were neutral or negative.\n- Protected/probation split: irrelevant at this cache size, so drop further sizing work.\n- Aging and doorkeeper: both lost on workload-a. They are untested on drifting workloads.\n"
}
```

### 19:56:27 chat

I wrote `cross-cycle-insights.md` to the cycle 0004 cross-cycle output directory. It has the four required sections in order. It is built only from the four ledger digests, and I haven't re-read it since writing it.

- **Recurring wins:** The only kind of change that has helped keeps frequency information and loosens admission slightly, and even that gain is tiny. Winners across cycles are ties or near-noise gains.
- **Recurring failures:** Anything that discards frequency information lost hit-rate. For four cycles the fitness output has also reported hit-rate only, so no cost trade has been judged.
- **Open questions:** Fixing the evaluation comes first: add metadata-size and per-op-time, plus a second trace. After that come aging on a shifting trace, whether single-hash indexing is actually faster, and a gentler sketch-width cut.
- **Per-policy notes:** These are grouped by sketch, window, protected/probation split, and aging/doorkeeper.


---

- finished: 2026-10-20T19:56:28+00:00
- duration: 15.24s
- status: ok
- input_tokens: 4
- output_tokens: 1400
- cache_read_tokens: 36527
- cache_write_tokens: 8151
- cost_usd: 0.050689
