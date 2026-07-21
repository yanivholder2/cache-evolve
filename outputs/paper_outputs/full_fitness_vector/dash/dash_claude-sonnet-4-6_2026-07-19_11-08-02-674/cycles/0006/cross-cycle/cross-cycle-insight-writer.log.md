# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0006\cross-cycle
- allow_edits: True
- started: 2026-07-20T17:07:37+00:00

## timeline

### 17:07:41 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. I need to find the output directory first, then write the file with the four fixed sections.

Let me look for the output directory structure.

### 17:07:42 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 17:07:42 tool: Glob

```json
{
  "pattern": "output/**"
}
```

### 17:07:44 tool: Bash

```json
{
  "command": "ls",
  "description": "List files in current working directory"
}
```

### 17:07:48 tool: Read

```json
{
  "file_path": "/Users/yaniv/Projects/cache-evolve/cross-cycle-insight-writer.log.md"
}
```

### 17:07:48 tool: Read

```json
{
  "file_path": "/Users/yaniv/Projects/cache-evolve/task.md"
}
```

### 17:07:50 thinking

The output directory is `C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0006\cross-cycle`. I need to write the file there.

### 17:08:29 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0006\\cross-cycle\\cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **MRU-end insertion is a stable anchor.** Flipping insertion from LRU-end to MRU-end improved hit rate and per-op latency in Cycle 1 and has held as a base for every subsequent winning variant. No descendant has regressed by inheriting it.\n- **Struct/metadata pruning yields free dividends.** Removing the `Instant` timestamp field reduced per-item footprint, eliminated clock-read syscalls, and improved bucket-traversal cache-line utilization — a pattern that has recurred wherever dead-weight fields were audited. Structural lightness amplifies every other mechanism layered on top.\n- **Orthogonal wins stack additively.** Mechanisms touching distinct subsystems (insertion position, struct layout, promotion gating, eviction selection) have consistently combined without negative interaction when properly inherited. The stacking hypothesis from Cycle 1 has been confirmed through Cycle 6.\n- **Vacancy-gated promotion eliminates cascade evictions by construction.** Guarding frequency-promotion fires on probing-tier vacancy suppressed the tail-latency regressions that plagued threshold-only approaches, making it a structurally sound gating primitive that has survived multiple stacking cycles.\n- **Two-chance clock eviction solves frequency-awareness without O(n) cost.** By rotating warm items (non-zero counter) to MRU-end and stopping at the first cold item, the clock achieves LFU-like victim selection while terminating early on Zipfian workloads — delivering hit-rate gains with latency improvements rather than regressions.\n\n## Recurring failures\n\n- **O(n) eviction scans reliably inflate tail latency.** Every variant that scanned a full bucket for the global LFU minimum incurred a measurable put-p99 regression regardless of what else the variant changed. The cost is consistent enough to treat as a structural contraindication for full-bucket scans on the hot eviction path.\n- **Threshold-only frequency gating cannot fully suppress cascade costs.** Raising the reuse counter threshold softens latency regressions but does not cure them, because the root problem is architectural: a gate that fires into a full tier will displace items regardless of firing rate. Cycle 3 and 4 both confirmed that structural vacancy checks outperform threshold calibration.\n- **Counter decay is antagonistic to frequency-gated promotion on stationary workloads.** Post-eviction counter halving (Cycle 6 v002) suppressed legitimately earned counters below the ≥ 2 threshold gate and added a second O(n) scan, doubling median latency. On a fixed popularity distribution there is no stale-popularity drift to correct, so decay has been net harmful every time it appeared.\n- **Overlay coverage gaps silently revert parent gains.** When a child hypothesis modifies only a subset of the files changed by its multi-file parent, missing files revert to baseline silently. Cycle 4 attributed regressions to the hypotheses under test when the actual cause was incomplete inheritance of the parent's struct reduction.\n- **Stash-bypass fires rate is base-dependent.** Stash-bypass was a hit-rate win on the LRU-end base but a regression on the MRU-end base because back-insertion makes target saturation far more common, amplifying bypass-fire rate and converting the probing tier into a spillover sink. Context sensitivity makes this idea unreliable without a vacancy guard.\n\n## Open questions\n\n- Can the two-chance clock (Cycle 6 v003) be stacked with the vacancy-gated target promotion and probing re-entry mechanisms from Cycle 6 v001, and does the combined variant preserve the clock's latency advantage while capturing additive hit-rate gains?\n- Is stash-bypass salvageable on the MRU-end base if guarded by a probing-vacancy check, mirroring the fix that made vacancy-gated promotion safe in Cycle 3?\n- Are there remaining struct fields or per-operation costs in the baseline analogous to `timestamp` — metadata that provides no eviction-selection advantage — that could yield another footprint/latency free win?\n- Does the vacancy gate's hit-rate benefit grow at higher cache occupancy (smaller cache relative to working set), where probing vacancies are rarer and gate-fires are more selectively throttled?\n- On a non-stationary workload where popularity drifts over time, does counter decay become net beneficial, and is there a guard condition (e.g., decay only when eviction rate exceeds a threshold) that makes it safe to combine with frequency-gated promotion?\n- Would the probing re-entry path contribute more hit-rate lift on workloads with higher probing residency, where demoted-but-reused items accumulate in probing at higher rates?\n\n## Per-policy notes\n\n- **LRU (positional, baseline):** Pure LRU victim selection serves as the floor. It is fast but blind to reuse frequency, allowing warm items to be displaced when they land at the LRU-end of a full bucket — the exact failure mode that MRU-end insertion and frequency-aware eviction variants both exploit.\n- **LFU-minimum (O(n) scan):** Confirmed to improve hit rate as a standalone mechanism (Cycle 5 v003) and as a stack component (Cycle 5 v001, Cycle 6 v001). The hit-rate gain is real and isolated to eviction selection. The consistent flaw is per-op tail latency from the full-bucket scan; the mechanism is viable only if the scan cost can be eliminated.\n- **Two-chance clock (Cycle 6 v003):** Currently the strongest eviction policy in the lineage. Approximates LFU victim selection in O(1) amortized cost by rotating warm items one pass and evicting the first cold item found. On a Zipfian workload the cold tail accumulates quickly, so early termination is the common case. Combines timestamp elimination, MRU-end insertion, and frequency awareness into a single regression-free win across all three measured dimensions.\n- **Vacancy-gated frequency promotion:** Not an eviction policy per se, but directly controls which items enter the promotion path. Pairing it with two-chance clock eviction is the natural convergence of the two most successful mechanism families identified to date.\n- **Counter decay (Cycle 6 v002):** Behaves as a policy modifier that reduces effective counter values post-eviction. On this stationary trace it conflicts with the ≥ 2 promotion gate and adds scan overhead; its applicability appears workload-distribution-dependent and should not be combined with frequency-gated promotion without a non-stationary trace to validate it against.\n"
}
```

### 17:08:37 chat

`cross-cycle-insights.md` written to the cycle 0006 cross-cycle output directory. Summary of what was synthesized across all six cycles:

**Recurring wins** captures the five durable patterns: MRU-end insertion as a persistent anchor, struct pruning (timestamp removal) as a free-dividend mechanism, confirmed additive stacking of orthogonal changes, vacancy-gated promotion as the structurally sound cascade-prevention primitive, and the two-chance clock as the resolved solution to frequency-aware eviction without O(n) cost.

**Recurring failures** documents five consistently anti-productive patterns: O(n) eviction scans inflating tail latency every time, threshold-only frequency gating failing to cure cascade costs architecturally, counter decay being antagonistic to the promotion gate on stationary workloads, overlay coverage gaps silently reverting parent gains (the Cycle 4 lesson), and stash-bypass being base-dependent and unreliable without a vacancy guard.

**Open questions** carries forward the six live threads most worth probing next, led by the natural convergence experiment: grafting v001's vacancy gates onto v003's clock base.

**Per-policy notes** characterizes each eviction policy (LRU baseline, LFU-minimum scan, two-chance clock, vacancy-gated promotion, counter decay) by its observed behavior profile across the cycles.


---

- finished: 2026-07-20T17:08:38+00:00
- duration: 60.75s
- status: ok
- input_tokens: 6
- output_tokens: 2417
- cache_read_tokens: 93378
- cache_write_tokens: 9163
- cost_usd: 0.10368
