# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0007\cross-cycle
- allow_edits: True
- started: 2026-07-20T17:35:19+00:00

## timeline

### 17:35:23 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. I need to find the output directory first, then write the file with exactly the four sections specified.

Let me first find where the output directory is located.

### 17:35:23 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 17:35:23 tool: Glob

```json
{
  "pattern": "**/output*"
}
```

### 17:35:26 tool: Bash

```json
{
  "command": "ls /",
  "description": "List root directory"
}
```

### 17:35:28 tool: Glob

```json
{
  "pattern": "**/*.md"
}
```

### 17:35:29 tool: Read

```json
{
  "file_path": "/task.md"
}
```

### 17:35:31 thinking

Now I know the allowed paths. The output directory seems to be `C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0007\cross-cycle`. Let me write the file there.

### 17:35:32 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0007\\cross-cycle\\task.md"
}
```

### 17:35:54 thinking

Now I have all the information I need. Let me write the cross-cycle-insights.md file in the output directory: `C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0007\cross-cycle\cross-cycle-insights.md`

Let me synthesize the seven cycles of data into the four required sections.

**Recurring wins:**
1. MRU-end insertion — confirmed win in Cycle 1, held through all subsequent cycles as a stable anchor
2. Timestamp/struct field removal — dual dividend (smaller footprint + eliminated syscalls), validated across cycles as a free win
3. Orthogonal wins stack additively — confirmed in Cycle 2, held through Cycle 6
4. Vacancy-gated promotion — structural elimination of cascade evictions, regression-free hit-rate lift, validated in Cycles 3–6
5. Two-chance clock eviction — frequency-aware eviction without O(n) scan cost, structural win in Cycle 6

**Recurring failures:**
1. Threshold-only calibration (no vacancy guard) — raises threshold doesn't cure latency regression, it's architectural not parametric
2. Stash-bypass on MRU-end base — reversed when base changed, fires too often under MRU insertion pressure
3. Joint condition mechanisms — requiring multiple conditions simultaneously tends to create brittle interactions that rarely trigger on this workload
4. Counter decay — antagonistic to frequency-gated promotion on stationary workloads, adds O(n) scan overhead
5. Clock + frequency gate sharing lfu_counter — fundamental invariant conflict: clock needs counter-zero cold items, promotion gate needs non-zero counters
6. Probing re-entry vacancy gate — persistently thin across Cycles 5, 6, 7 at high cache occupancy

**Open questions:**
1. Can clock eviction and frequency-gated promotion coexist via a separate "chance bit" decoupled from lfu_counter?
2. Can conditional counter increment on target access be gated to fire only on high-eviction-risk items to preserve the hit-rate signal without common-path overhead?
3. Does the workload have specific patterns that could guide which mechanism to prioritize?
4. Are there other struct fields or per-op overhead sources beyond timestamp still worth auditing?
5. At what cache occupancy / working-set ratio does probing re-entry vacancy gate produce a useful firing rate?

Looking at the per-policy tradeoffs: LRU provides a baseline, LFU-minimum improves hit-rate but consistently inflates put p99 latency due to scan costs, and two-chance clock avoids the O(n) scan problem by rotating through items instead.

Two-chance clock wins across all three dimensions in Cycle 6, but it conflicts with frequency-gated promotion when they share the counter field — the clock's decrements corrupt the frequency signal. Frequency-gated promotion with a ≥2 threshold has been stable since Cycle 3 and appears near-optimal for this workload, though it's incompatible with both counter decay and the two-chance clock's counter semantics. Counter decay itself proved harmful on stationary workloads, doubling median latency through a second O(n) scan and suppressing counters below the promotion threshold.

### 17:36:15 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0007\\cross-cycle\\cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **MRU-end insertion** has been a stable anchor since Cycle 1 — a single placement flip improved both hit-rate and per-op time simultaneously and has held through all subsequent cycles without regression.\n- **Struct field removal (timestamp)** delivered a dual dividend (smaller per-item footprint, eliminated syscalls) with zero hit-rate cost; similar overhead audits are likely to yield further free wins.\n- **Orthogonal improvements stack additively** when they touch non-overlapping subsystems — confirmed for insertion position × struct compaction (Cycle 2) and for eviction selection × promotion gating (Cycles 5–6).\n- **Vacancy-gated frequency promotion (≥2 threshold)** has been regression-free since Cycle 3 and survived stacking through Cycle 6; gating promotion fires on tier vacancy is the structural fix that eliminates cascade evictions by construction.\n- **Two-chance clock eviction** (Cycle 6) solved the long-standing O(n) scan latency problem: rotating warm items to MRU end and stopping at the first cold item delivers frequency-aware victim selection with amortized O(1) cost and improved latency on a Zipfian workload.\n\n## Recurring failures\n\n- **Threshold-only calibration without a vacancy guard** consistently softens but cannot cure the latency regression from promotion cascades; the root cause is architectural (firing into a full tier), not parametric.\n- **Stash-bypass on the MRU-end base** reversed from a win (LRU-end base, Cycle 2) to a regression (MRU-end base, Cycle 3) because MRU insertion makes target saturation the common case, over-triggering the bypass and converting probing into a spillover sink.\n- **Joint-condition mechanisms that require two rare simultaneous events** (e.g., target-full AND probing-vacant at stash-hit time) fire too infrequently on this workload to produce a measurable effect — confirmed inert in Cycle 4.\n- **Counter decay** is antagonistic to frequency-gated promotion on a stationary workload: it suppresses counters below the promotion threshold and adds a second O(n) scan at eviction time, doubling median latency (Cycle 6).\n- **Merging clock eviction with frequency-gated promotion via the shared `lfu_counter`** fails structurally: the clock requires counter-zero cold items to exit early; the promotion gate requires non-zero counters to identify warm items. When the bucket is counter-rich, the clock degrades to O(k·n) rotation and erases its latency advantage (Cycle 7).\n- **Vacancy-gated probing re-entry** is structurally safe but persistently thin at high cache occupancy (Cycles 5, 6, 7) — the joint firing condition is too rare under the Zipfian steady-state where hot items cycle through probing continuously.\n\n## Open questions\n\n- Can the two-chance clock and frequency-gated promotion coexist if their signals are decoupled — e.g., a separate boolean \"chance bit\" for the clock that leaves `lfu_counter` untouched for promotion gates?\n- Can the confirmed hit-rate signal from counter-increment on target access (H03) be captured cheaply by gating the increment only when an item is near eviction risk, avoiding the common-path overhead that tripled median latency in Cycle 7?\n- Are there additional struct fields or per-operation overhead sources beyond `timestamp` still worth auditing in the current baseline?\n- Does counter decay become net-positive on a non-stationary workload where stale-popularity drift is a real problem, and can a firing-rate guard make it safe to combine with frequency gates?\n- At what cache-to-working-set ratio does the probing re-entry vacancy gate fire often enough to produce a measurable hit-rate contribution?\n\n## Per-policy notes\n\n- **Positional LRU (baseline)**: MRU-end insertion was a direct improvement from Cycle 1 and has been the default since; no further LRU-order wins have appeared.\n- **LFU-minimum eviction (O(n) scan)**: Independently confirmed as a hit-rate lever in Cycles 5–6 and additive with vacancy-gated promotion, but consistently inflated put p99 due to scan cost. Superseded by the two-chance clock as the preferred frequency-aware eviction strategy.\n- **Two-chance clock eviction**: Structurally improved all three measured dimensions in Cycle 6 from baseline. Incompatible with frequency-gated promotion when sharing `lfu_counter` — clock decrements corrupt the promotion signal (Cycle 7). Decoupling the chance bit from the frequency counter is the unresolved prerequisite for composing these two mechanisms.\n- **Frequency-gated vacancy-guarded promotion (≥2)**: Regression-free hit-rate lift since Cycle 3; ≥2 appears near-optimal for this workload (≥3 produced a net loss in Cycle 4). Survives stacking with LFU-minimum eviction and probing re-entry (Cycles 5–6) but conflicts with both counter decay and the two-chance clock when `lfu_counter` is shared.\n- **Counter decay**: Single test in Cycle 6; harmful on a stationary distribution — doubles median latency and suppresses the frequency signal. Should be reserved for non-stationary workloads where popularity shifts are a real operational concern.\n"
}
```

### 17:36:30 chat

Written `cross-cycle-insights.md` to the output directory. Here's a summary of what the file captures across all seven cycles:

**Recurring wins** — Five durable patterns: MRU-end insertion (stable anchor since Cycle 1), struct field removal as a free dividend, additive stacking of orthogonal changes, vacancy-gated frequency promotion (regression-free since Cycle 3), and the two-chance clock eviction (Cycle 6's structural solution to the O(n) scan problem).

**Recurring failures** — Six patterns that keep not paying off: threshold-only cascade suppression (architectural, not parametric), stash-bypass on the MRU-end base (reversed due to insertion-pressure interaction), joint rare-event conditions (fire too infrequently), counter decay on stationary workloads, clock+promotion sharing `lfu_counter` (fundamental invariant conflict discovered in Cycle 7), and vacancy-gated probing re-entry (persistently inert at high occupancy across three consecutive cycles).

**Open questions** — Five threads, led by the `lfu_counter` decoupling question (separate chance bit vs. promotion counter) as the blocker for composing the clock with frequency gates.

**Per-policy notes** — Organized by policy lineage: positional LRU, LFU-minimum scan (superseded), two-chance clock (current eviction anchor, pending decoupling fix), vacancy-gated promotion (current promotion anchor, incompatible with clock as-is), and counter decay (single test, deprioritized).


---

- finished: 2026-07-20T17:36:30+00:00
- duration: 71.71s
- status: ok
- input_tokens: 8
- output_tokens: 3205
- cache_read_tokens: 144421
- cache_write_tokens: 16354
- cost_usd: 0.158561
