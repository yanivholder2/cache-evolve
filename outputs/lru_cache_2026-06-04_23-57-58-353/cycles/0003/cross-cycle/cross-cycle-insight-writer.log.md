# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-06-04T21:52:29+00:00

## timeline

### 21:52:33 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. Let me first find the output directory, then write the file.

Let me look for the output directory structure.

### 21:52:33 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 21:52:33 tool: Glob

```json
{
  "pattern": "output*/**"
}
```

### 21:52:36 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/"
}
```

### 21:52:38 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve"
}
```

### 21:52:42 thinking

The allowed roots tell me:
1. `/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/cross-cycle`
2. `/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles-ledger.jsonl`


I'll write the output file to that cross-cycle directory.

### 21:52:42 tool: Glob

```json
{
  "pattern": "*",
  "path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/cross-cycle"
}
```

### 21:53:10 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Two-segment SLRU as a foundation**: Inserting new entries into a probationary segment before they can displace proven hot entries has delivered durable gains every cycle it appeared. Acting at insertion time (rather than eviction time) is the root of this advantage.\n- **Scan-fraction sizing over fixed counts**: Mechanisms that scale their working-set scan to a proportional share of the segment (e.g., one quarter of protected) consistently outperform those using a small fixed count, because the signal stays meaningful as capacity grows.\n- **Counter halving over counter reset on demotion**: Preserving a decayed form of accumulated frequency history across segment transitions allows promoted entries to remain distinguishable from newcomers — resetting to zero throws away a durable and informative signal.\n- **Position-update discipline on protected hits**: Every mechanism that maintained continuous move-to-tail on access in the protected segment produced better ordering; suppressing position updates for any reason has consistently degraded hit rates.\n\n## Recurring failures\n\n- **Eviction-time-only filters**: Both the CLOCK bit-scan (cycle 1) and the fixed-window frequency scan (cycle 1) acted only when eviction was forced, missing entries that should have been deprioritized much earlier. Eviction-time intervention is weaker than insertion-time or promotion-time routing.\n- **Small fixed frequency windows**: A constant small look-ahead (e.g., 8 entries) leaves a structural ceiling on improvement regardless of how accurate the frequency signal is within that window.\n- **Ghost admission on this workload**: Tested in isolation in cycle 3 after cycle 2's ambiguity, the ghost set produced negligible lift. The probationary-capacity ghost window is too narrow to intercept reuse gaps in the current trace; the improvement originally attributed to it was SLRU segmentation.\n- **Protected-segment widening past 80/20**: Shrinking the probationary share to 30% triggered more demotion churn than the extra protected headroom could absorb; the hot working set is large enough that capacity allocation shifts hurt more than they help.\n- **Second-chance reprieve with position freeze**: Granting a one-time demotion reprieve while freezing node position severed the recency ordering that makes LRU useful. Hot entries stalled near the demotion head regardless of their reprieve status.\n\n## Open questions\n\n- Does frequency-aware demotion interact cleanly with ghost admission, or does the ghost set introduce noise now that both operate simultaneously in the same pipeline?\n- Is the LRU-quarter scan fraction robust across different workload shapes, or does the right fraction shift with reuse-distance distributions?\n- Could a significantly larger ghost window (e.g., equal to total capacity rather than probationary capacity) make cross-residency warmth visible enough to matter on this trace, or is the ghost concept fundamentally mismatched to the workload?\n- Could a second-chance approach be rehabilitated by setting the referenced bit while still performing move-to-tail, so recency ordering is preserved alongside the reprieve?\n- Does CLOCK's linked-list-free structure yield measurable throughput gains under high concurrency or at very large capacities, separate from its hit-rate characteristics?\n\n## Per-policy notes\n\n- **Plain LRU (baseline)**: Scan-pollution vulnerability is confirmed real and addressable by multiple independent mechanisms. All variants in cycle 1 improved on it.\n- **SLRU (v001, cycle 1 winner)**: Probationary-to-protected promotion is the load-bearing mechanism. The 80/20 protected/probationary split performed best; both narrower and wider protected segments degraded results. This structure has served as the base for all subsequent winners.\n- **CLOCK (cycle 1)**: Eliminates pointer maintenance overhead; hit-rate gains were real but smaller than SLRU because eviction-time intervention is weaker. Throughput advantage under concurrency remains untested.\n- **Frequency-window eviction (cycle 1 v003, cycle 2 v002 retry)**: Cycle 1's fixed-8-slot version had a structural ceiling. Cycle 3's redesign — proportional scan window plus counter halving — became the winner, confirming the idea is sound once signal-design flaws are corrected.\n- **Ghost admission (cycle 2 v003, cycle 3 v001 isolation)**: Appeared effective in cycle 2 but was confounded with SLRU. When isolated in cycle 3, contribution was negligible on this workload. Not worth pursuing unless workload shifts toward a trace with frequent cross-residency reuse.\n- **Frequency-aware demotion with counter halving (cycle 3 winner)**: Now the strongest single add-on to SLRU. Scanning the LRU quarter of protected and halving (not resetting) counters on demotion are both required for the mechanism to work; either alone was insufficient in earlier designs.\n"
}
```

### 21:53:16 chat

Written to `cross-cycle-insights.md`. Here's a summary of what was synthesized across the three cycles:

**Recurring wins** — SLRU's insertion-time probationary routing, proportional scan-fraction sizing, counter halving over resets, and uninterrupted move-to-tail on protected hits all showed up as durable contributors across multiple cycles.

**Recurring failures** — Eviction-time-only filters (CLOCK, fixed frequency windows), ghost admission on this workload, widening probationary beyond the 80/20 split, and second-chance reprieve with position freezing all showed consistent non-performance, with root causes now clearly identified.

**Open questions** — Flags the frequency-demotion + ghost interaction, scan-fraction robustness across workload shapes, larger ghost windows, a rehabilitated second-chance design, and CLOCK's throughput story under concurrency.

**Per-policy notes** — Grouped observations by policy (baseline LRU, SLRU, CLOCK, frequency-window, ghost admission, frequency-demotion+halving) to give the hypothesis writer a compact per-mechanism reference.


---

- finished: 2026-06-04T21:53:17+00:00
- duration: 47.34s
- status: ok
- input_tokens: 7
- output_tokens: 2100
- cache_read_tokens: 91743
- cache_write_tokens: 6682
- cost_usd: 0.086684
