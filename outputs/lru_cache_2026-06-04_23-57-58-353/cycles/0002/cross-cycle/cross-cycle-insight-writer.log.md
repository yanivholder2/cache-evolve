# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-06-04T21:41:01+00:00

## timeline

### 21:41:04 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. I need to find the output directory first, then write the file with the four fixed sections.

Let me look for the output directory structure.

### 21:41:04 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 21:41:04 tool: Glob

```json
{
  "pattern": "output*/**"
}
```

### 21:41:07 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/"
}
```

### 21:41:09 tool: Glob

```json
{
  "pattern": "**/*"
}
```

### 21:41:11 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/cross-cycle-insight-writer.log.md"
}
```

### 21:41:11 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/task.md"
}
```

### 21:41:14 thinking

The allowed output directory is `/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/cross-cycle`. I'll write the `cross-cycle-insights.md` file there.

### 21:41:14 chat

The output directory is `/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/cross-cycle`. Now I'll synthesize the two-cycle ledger into the four fixed sections and write the file.

### 21:41:34 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Two-segment SLRU** has been the most durable structural improvement: routing new entries into a probationary buffer before they can displace the protected working set consistently outperforms plain LRU and has held up across both cycles.\n- **Acting at insertion time beats acting at eviction time.** Every strategy that intercepts entries before they enter the hot set (SLRU, ghost admission) outperformed strategies that only act when capacity is exceeded (CLOCK, frequency-window scan).\n- **Ghost admission on top of segmentation** extended gains in cycle 2 by capturing cross-residency warmth — keys evicted from probationary that re-arrive earn fast promotion into protected, which SLRU alone does not track.\n\n## Recurring failures\n\n- **Frequency-aware demotion inside the protected segment** has failed in the one trial attempted (cycle 2). The root cause is a signal-design flaw: a narrow scan window covering only a fraction of the segment and counters that reset to zero on every demotion starve the mechanism of durable signal before it can discriminate high-reuse from low-reuse entries. This is not yet evidence that frequency is uninformative — only that the current implementation is too weak.\n- **Widening probationary beyond the default split** hurt in cycle 2. The hot working set is large enough that shrinking the protected segment below the 80/20 baseline causes more demotion churn than the extra scan-filtering headroom saves. Ratio tuning appears to be a diminishing-return axis and is likely not worth further exploration at current workload scale.\n\n## Open questions\n\n- **Ghost admission contribution in isolation.** v003 (cycle 2) was built from the plain-LRU baseline, so its gain bundles SLRU segmentation and ghost admission together. Applying ghost admission on top of the established cycle-1 SLRU winner would isolate the ghost set's marginal value.\n- **Frequency-aware demotion with a corrected design.** Would persistent (non-resetting) counters and a much larger scan window allow frequency demotion to recover? The idea is not exhausted — only the current implementation is ruled out.\n- **Optimal ghost window size.** Ghost capacity equal to probationary capacity is the only size tried. A smaller ghost window (e.g., half of probationary) might preserve most of the benefit at lower bookkeeping cost.\n- **CLOCK throughput advantage.** CLOCK eliminates linked-list pointer updates entirely. Under high concurrency or very large capacities its structural simplicity may produce measurable throughput gains even if its hit-rate ceiling is lower; this has not been tested.\n\n## Per-policy notes\n\n- **SLRU (two-segment):** Consistent winner across cycles 1 and 2 in multiple configurations. The 80/20 protected-to-probationary split appears to be a local optimum for the scan-heavy workload tested; looser splits degrade it. It is now the established baseline.\n- **Ghost admission (LIRS-style):** Won cycle 2 as the v003 variant. Effective when traces contain recurring keys that cycle in and out of cache (re-arrival after eviction is a reliable warmth signal). Contribution not yet isolated from SLRU segmentation.\n- **CLOCK:** Produced a hit-rate improvement in cycle 1 but trailed SLRU. Its eviction-time-only signal is weaker than SLRU's insertion-time partitioning. Throughput under concurrency remains an unexplored upside.\n- **Frequency-window scan (eviction-time):** Bounded by a fixed look-ahead in cycle 1; frequency-aware demotion inside protected failed in cycle 2. Both failures trace to the same root: frequency signal is not durable enough when the observation window is small and counters reset. Needs a redesign before further trials.\n"
}
```

### 21:41:41 chat

`cross-cycle-insights.md` written to the cycle-0002 cross-cycle output directory. Here's a summary of what was synthesized across the two completed cycles:

**Recurring wins** — Two-segment SLRU is the durable structural core: its insertion-time partitioning consistently beats eviction-time-only strategies. Ghost admission extends SLRU's gains by capturing cross-residency warmth.

**Recurring failures** — Frequency-aware demotion has failed in both placements tried (eviction-time scan with a fixed window in cycle 1; in-protected demotion with resetting counters in cycle 2). The failure mode is always signal starvation, not a fundamental limit. Widening probationary beyond the 80/20 split also regressed.

**Open questions** — Ghost admission's marginal contribution is still unquantified (it was bundled with SLRU in cycle 2). Frequency demotion deserves a redesign with persistent counters and a wider window before being abandoned. CLOCK's throughput advantage under concurrency is unexplored.

**Per-policy notes** — Groups observations by SLRU, ghost admission, CLOCK, and frequency-window scan, calling out each policy's observed ceiling and the specific design flaws responsible for failures.


---

- finished: 2026-06-04T21:41:42+00:00
- duration: 40.78s
- status: ok
- input_tokens: 7
- output_tokens: 1864
- cache_read_tokens: 87963
- cache_write_tokens: 5855
- cost_usd: 0.078168
