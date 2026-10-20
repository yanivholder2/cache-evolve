# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\cross-cycle
- allow_edits: True
- started: 2026-07-19T09:23:41+00:00

## timeline

### 09:23:46 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. I need to find the output directory first, then write the file with the four fixed sections.

Let me first find where the output directory is located.

### 09:23:46 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 09:23:46 tool: Glob

```json
{
  "pattern": "**/output*"
}
```

### 09:23:48 tool: Bash

```json
{
  "command": "ls",
  "description": "List current working directory"
}
```

### 09:23:56 tool: Read

```json
{
  "file_path": "/Users/yaniv/Projects/cache-evolve/cross-cycle-insight-writer.log.md"
}
```

### 09:23:57 tool: Read

```json
{
  "file_path": "/Users/yaniv/Projects/cache-evolve/task.md"
}
```

### 09:24:06 thinking

Now I know the output directory: `C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\cross-cycle`

I should write `cross-cycle-insights.md` there. Let me synthesize the insights from the three cycles.

Key patterns across cycles:
1. Structural changes that eliminate cascade evictions outperform threshold tuning
2. Stacking independent wins is additive when they touch separate subsystems
3. MRU-end insertion changed tier dynamics globally
4. Struct/metadata overhead removal (timestamp) is free wins
5. Vacancy-gating emerged as the cleanest improvement

Threshold-only calibration keeps hitting a wall with latency regressions, and stash-bypass flipped when I switched to the MRU-end base. The LFU hypothesis still timed out without revealing whether it's viable. I'm curious about combining vacancy gating with threshold adjustments, testing stash-bypass under a vacancy guard on the MRU base, hunting for other dead-weight fields beyond the timestamp, and finally resolving what happened with LFU. LRU benefited most from the MRU-end insertion boost to hit rates, while LFU's operational timeout in cycle 1 left its merit unknown.

Per-operation timing shows frequency-based policies care more about structure than threshold alone.

### 09:24:06 chat

Now I have the output directory path. Let me write the file:

### 09:24:26 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\cross-cycle\\cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Structural cascade elimination beats threshold tuning.** Across cycles 2–3, every attempt to suppress cascade evictions via rate-limiting (threshold tuning) left a residual latency regression, while architectural gating (vacancy check before firing) produced clean, regression-free hit-rate gains. The pattern is reliable enough to treat as a design principle.\n- **Stacking orthogonal wins is additive.** Cycle 2 confirmed that MRU-end insertion and timestamp removal — which touch entirely separate subsystems — compose without interaction. Combining verified independent wins before adding new ideas has been a consistently safe strategy.\n- **Struct/metadata audits yield free wins.** Timestamp removal delivered smaller memory footprint and eliminated syscalls with zero hit-rate cost. The baseline appears to carry further dead-weight overhead that has not yet been audited.\n- **MRU-end insertion is a durable foundation.** The insertion-position flip has been the starting point for every winning variant since cycle 1 and has not been dethroned by any subsequent change.\n\n## Recurring failures\n\n- **Threshold-only calibration cannot cure cascade latency.** Raising the frequency gate from ≥ 2 to ≥ 3 narrowed but did not eliminate the p99 regression. A gate that can fire into a full tier will always cause cascades regardless of firing frequency; tuning the rate is treating a symptom, not the cause.\n- **Stash-bypass without a vacancy guard reverses on the MRU-end base.** The bypass won on the LRU-end base (cycle 2) but regressed on the MRU-end base (cycle 3) because back-insertion makes target saturation the common case, dramatically increasing bypass-fire rate. The mechanism is not portable across insertion-position regimes without a structural guard.\n- **LFU forced-eviction remains untested at meaningful scale.** The hypothesis has been discarded twice — once by simulator timeout, once by deprioritization — without ever producing a fitness signal. It cannot be confirmed or ruled out yet.\n\n## Open questions\n\n- Can the vacancy gate and a threshold of ≥ 3 be combined to restrict promotion fires to items with both open probing slots and demonstrated reuse, pushing hit-rate higher than either alone?\n- Is stash-bypass salvageable on the MRU-end base if gated on probing vacancy, mirroring the structural fix that made vacancy-gated frequency promotion succeed?\n- Are there additional struct fields or per-operation costs in the baseline — analogous to `timestamp` — that carry no eviction advantage on this workload?\n- Does the vacancy gate's hit-rate lift grow at higher cache occupancy, where probing vacancies are rarer and gate-fires are naturally throttled further?\n- Is the LFU eviction hypothesis worth testing on a lighter trace or smaller cache to finally get a fitness signal before the idea is permanently set aside?\n\n## Per-policy notes\n\n- **LRU / MRU-end insertion:** The single highest-leverage change across all cycles. Flipping insertion from LRU-end to MRU-end simultaneously improved hit rate and per-op time by aligning placement with the tier hierarchy's warm-up logic. Every subsequent winner has built on this.\n- **Frequency-based promotion (frequency gate):** The underlying signal is real — the workload contains re-accessed non-hottest items — but the mechanism is sensitive to tier fullness. At threshold ≥ 2 the cascade cost dominates; at ≥ 3 it is softened but not eliminated. Vacancy-gating is the structurally sound path forward for this policy.\n- **Stash-bypass:** A clean hit-rate win under LRU-end insertion; becomes a regression under MRU-end insertion due to amplified bypass-fire rate. Should be treated as insertion-position-dependent and retried only with a vacancy guard.\n- **LFU forced eviction:** Timed out in cycle 1, never retried. No fitness signal available; eviction merit is genuinely unknown.\n"
}
```

### 09:24:32 chat

Written `cross-cycle-insights.md` to the output directory. Here's a summary of what was synthesized across the three cycles:

**Recurring wins** — MRU-end insertion remains the durable foundation; stacking orthogonal wins is additive; vacancy-gating (structural cascade elimination) consistently outperforms rate-limiting; struct/metadata audits have yielded free gains without regressions.

**Recurring failures** — Threshold-only calibration cannot cure cascade latency (treating a symptom, not the cause); stash-bypass reverses on the MRU-end base without a vacancy guard; LFU has been discarded twice without ever producing a fitness signal.

**Open questions** — Whether combining vacancy gate + threshold ≥ 3 compounds hit-rate; whether stash-bypass becomes viable with a vacancy guard on the MRU base; whether more dead-weight struct fields exist beyond `timestamp`; whether LFU deserves one final lightweight retry.

**Per-policy notes** — Captures the insertion-position sensitivity of stash-bypass, the real-but-structurally-unsound signal in frequency gates, and the still-unknown status of LFU.


---

- finished: 2026-07-19T09:24:32+00:00
- duration: 51.15s
- status: ok
- input_tokens: 6
- output_tokens: 2037
- cache_read_tokens: 83538
- cache_write_tokens: 6362
- cost_usd: 0.082052
