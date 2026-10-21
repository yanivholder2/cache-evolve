# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:20:23+00:00

## timeline

### 20:20:32 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Sizing is the only lever with consistent signal: a small queue shrunk from baseline (to roughly 2-5%) paired with a shortened ghost beats baseline. Gains arrive as small steps and flatten out.\n- Sizing-only configurations match or beat more complex bundles. The ghosting of main evictees was not needed for the gain, so prefer the simplest config.\n- Ghost length acts as an admission filter: shorter helps or is neutral, and longer has hurt in each test.\n- Consolidating earlier gains into one parent gave the cleanest wins. Single-rule tweaks mostly gave ties or small losses.\n\n## Recurring failures\n\n- Loosening admission (looser promotion, pre-protected or boosted ghost re-admits, longer ghost) hurts by polluting main.\n- Ageing and decay changes (faster freq decay, lower freq cap, halving, in-place ageing) have lost every time. The workload rewards keeping reuse history. A higher cap gave only a marginal gain.\n- Rewrites of main eviction (lookahead, reinsertion caps) lose. The unbounded second-chance scan is protective.\n- Filtering or restricting ghost hits or ghost contents (freq>0 gate, reuse-only ghosting) loses. Ghost hits are strong reuse evidence.\n- Promotion strictness and timing (eviction-time gates, hit-path promotion, thresholds, freq reset) repeatedly tie exactly. They are either not binding on this trace or the diffs are no-ops.\n- Adaptive or feedback-driven sizing lost to the fixed small ratio.\n- Per-key ghost metadata and main insertion-position tweaks gave no signal.\n\n## Open questions\n\n- Do exact ties reflect real no-ops in the code? Variant authors cannot read overlay source, so this is unverified.\n- Are the small gains (2% vs 3% vs 5%) real or noise? Sizing probes have never been compared head-to-head on one parent, and effective queue lengths may round to the same value.\n- Do any results hold on a second trace or cache size? All evidence is one trace at one cache size.\n- What do per-op-time and metadata-size look like? They have been unmeasured in every cycle, and ghost work and hit-path actions may add cost.\n- Is there any structurally new lever beyond sizing, or is S3-FIFO at a local optimum here?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy so far): the structure is near a local optimum on this trace. The best known base is the simple sizing-only config with a small queue of ~2% and a shortened ghost. Stop tuning ratio, ghost length, promotion thresholds, freq cap and ageing. Next steps are validation on a second trace and cache size, restoring cost metrics, and only structurally new levers whose diffs are verified to change behaviour.\n"
}
```

### 20:20:35 chat

I wrote `cross-cycle-insights.md` to the cycle 0010 cross-cycle output directory. It has the four required sections, in the required order, and covers cycles 0001–0010.

Its main conclusions:
- **Wins:** Sizing is the only lever with consistent signal. That means a smaller small queue (roughly 2–5%) paired with a shortened ghost. The gains are small and flatten out.
- **Failures:** Several approaches keep losing or tying. These are looser admission, ageing and decay changes, rewrites of main eviction, and filtering ghost hits. Promotion tweaks tie exactly every time, and adaptive sizing lost to the fixed ratio.
- **Open questions:**
  - Whether the exact ties are real or the diffs are no-ops.
  - Whether the small gains are noise.
  - Whether results hold on a second trace or cache size.
  - What the per-op-time and metadata-size costs are. These have never been measured.
- **Per-policy notes:** S3-FIFO looks near a local optimum on this trace. The suggested base is the simple sizing-only config with a ~2% small queue and a shortened ghost.


---

- finished: 2026-10-20T20:20:36+00:00
- duration: 13.18s
- status: ok
- input_tokens: 4
- output_tokens: 1475
- cache_read_tokens: 40552
- cache_write_tokens: 12189
- cost_usd: 0.068394
