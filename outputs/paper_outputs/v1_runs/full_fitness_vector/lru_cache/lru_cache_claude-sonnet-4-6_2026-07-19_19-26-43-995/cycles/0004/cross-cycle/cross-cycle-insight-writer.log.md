# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0004\cross-cycle
- allow_edits: True
- started: 2026-07-19T17:39:01+00:00

## timeline

### 17:39:05 thinking

Let me analyze the ledger digest across all 4 cycles and write the cross-cycle-insights.md file. I need to find the output directory first.

### 17:39:05 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 17:39:05 tool: Glob

```json
{
  "pattern": "**/output*/**"
}
```

### 17:39:09 tool: Bash

```json
{
  "command": "ls /c/Users/yaniv/Projects/cache-evolve/ 2>/dev/null || ls ~/Projects/cache-evolve/ 2>/dev/null || find /c/Users/yaniv -maxdepth 4 -name \"*.md\" 2>/dev/null | head -20",
  "description": "Explore project directory structure"
}
```

### 17:39:12 tool: Glob

```json
{
  "pattern": "**/*.md"
}
```

### 17:39:15 tool: Read

```json
{
  "file_path": "/c/Users/yaniv/Projects/cache-evolve/cross-cycle-insight-writer.log.md"
}
```

### 17:39:15 tool: Read

```json
{
  "file_path": "/c/Users/yaniv/Projects/cache-evolve/task.md"
}
```

### 17:39:18 thinking

Now I know the output path. I need to write to:
`C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0004\cross-cycle\cross-cycle-insights.md`

Let me analyze the 4 cycles of ledger data and write the insights.

### 17:39:18 chat

Now I have the allowed output path. Let me synthesize across all 4 cycles and write the file.

### 17:39:54 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0004\\cross-cycle\\cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Structural compaction of node metadata pays off reliably.** The u32-sentinel link encoding (Cycle 0001) halved node size and improved get latency through better cache-line utilization; this remains the unchallenged baseline. Encoding policy bits (queue-membership flag, SEEN_BIT) into spare fields of existing fields costs nothing in layout and has consistently been the right implementation approach.\n- **Meaningful behavioral changes outperform micro-optimizations.** Across every cycle, variants that made a genuine algorithmic change (2Q policy, MRU-tail guard) won or nearly won, while variants applying code-level hints (`#[inline(always)]`, `entry()` API, forced inlining) consistently failed. The instruction-cache and borrow-checker constraints in the hot path make micro-optimization fragile.\n- **Two-queue (2Q) policy improves hit-rate on Zipfian workloads.** The protected-bit-in-prev encoding kept Node at 24 bytes while delivering 2Q semantics (Cycle 0002 winner). Hit-rate gains from protecting hot items have appeared across multiple cycles even when the per-op cost moved unfavorably.\n\n## Recurring failures\n\n- **CLOCK-style eviction has failed twice with the same root cause.** In Cycle 0001, a CLOCK variant lost due to hand-sweep cache-line pollution. In Cycle 0004, a bitmap-backed CLOCK variant lost due to Vec header bloat and the same sweep cost at demotion time. The amortization argument for deferred relinking only holds when promotions vastly outnumber demotions — on a full cache these rates are coupled, making the trade-off structurally unfavorable.\n- **Single-path shortcuts with low fire rates are costly.** The MRU-tail guard won in Cycle 0002 on its native workload but failed when composed with 2Q in Cycle 0003, because it fires too rarely on workload-a to justify the unconditional check. The same failure mode appeared in Cycle 0004 with two-hit graduation: the splice was triggered on first probationary access (the common case) rather than second access (the rare graduation event).\n- **Adaptive and policy-response mechanisms are wasted on a stationary distribution.** The Cycle 0004 adaptive quota sizer never triggered a single adjustment epoch on the stable Zipfian trace, yet paid counter-increment and branch overhead on every operation. Any mechanism designed for workload phase shifts is invisible to this benchmark.\n- **API-level put-path tricks are blocked by borrow-checker constraints.** The `entry()` API optimization (Cycle 0002 loser) failed because the occupied `VacantEntry` must be released before `map.remove()` can run on the eviction path, erasing the promised single-probe benefit. This is a structural incompatibility, not a tuning issue.\n- **Policy-layer search space around 2Q appears largely exhausted.** Adaptive quota, CLOCK second-chance, MRU-tail guard, two-hit admission, protected-quota tuning, and 2Q+guard composition have all been tried; none produced a net winner across both hit-rate and latency on this workload.\n\n## Open questions\n\n- **Can the two-hit graduation restructuring recover the second-chance benefit?** Marking the SEEN_BIT without relinking on first probationary access (paying only a bitwise OR) and deferring the splice to the second access would move extra work to the rare path — the right calibration. This has not been tested.\n- **Do non-Copy key types finally validate the key-clone elision hypothesis?** The eviction-path refactor showed no effect under `Copy` keys. A `String`- or `Arc<str>`-keyed benchmark is needed to determine whether eliminating the clone on eviction is a real put-path win.\n- **Can a structural memory-layout change reduce node-access cache-miss cost without per-op branches?** Arena packing, slab grouping, or hot-field alignment have not been tried. This is the next orthogonal axis: a structural win might reopen headroom for policy experiments that currently can't overcome the tight per-op budget.\n- **Is there a workload where adaptive quota sizing earns its overhead?** On a bursty or phase-shifting trace the mechanism might actually trigger and produce a net hit-rate gain. The current benchmark's stationarity prevents answering this.\n\n## Per-policy notes\n\n- **LRU (baseline):** The u32-sentinel doubly-linked list is the structural baseline; 24-byte nodes. All subsequent work builds on this. Get latency is excellent when the hot path avoids extra branches.\n- **Two-queue (2Q):** Confirmed hit-rate improver on Zipfian workloads. Protected-bit-in-prev encoding keeps node at 24 bytes. Protected quota at 75% of capacity is the current default; expanding to 87.5% improves hit-rate but degrades throughput proportionally. Composing 2Q with the MRU-tail guard added cumulative overhead without net gain on workload-a.\n- **CLOCK:** Failed in two independent implementations (linked-list sweep and bitmap-backed). Deferred relinking does not amortize well when promotion and demotion rates are coupled at capacity. Deprioritize unless a specific mechanism to avoid hand-sweep cache pollution is proposed.\n- **Two-hit / second-chance:** Directionally correct (one-hit graduates do exist in the trace), but fire-rate calibration is wrong — the extra work currently lands on the common path. Restructuring to fire only at second access is the required next step before this can be evaluated fairly.\n- **MRU-tail guard:** Workload-dependent win. Fire rate too low on workload-a at steady state under 2Q. Do not revisit in isolation; only reconsider if workload characteristics change or fire-rate analysis shows a higher-hit-rate regime.\n"
}
```

### 17:40:01 chat

Written `cross-cycle-insights.md` to the cycle 0004 cross-cycle output directory. Here's a brief summary of what was synthesized across all four cycles:

**Recurring wins:** Node metadata compaction (u32 sentinels) is the durable structural baseline; meaningful behavioral changes consistently beat micro-optimization hints; 2Q policy reliably improves hit-rate on Zipfian workloads when node layout stays at 24 bytes.

**Recurring failures:** CLOCK has failed twice for the same root cause (sweep-induced cache-line pollution); single-path shortcuts with low fire rates pay unconditional overhead with negligible benefit; adaptive mechanisms are invisible on the stationary Zipfian distribution; `entry()` API tricks are blocked by borrow-checker eviction-path constraints; the policy-layer search space around 2Q is largely exhausted.

**Open questions:** Two-hit graduation restructured to fire only at second access (not first); key-clone elision validated against non-`Copy` key types; structural arena/slab layout changes as the next orthogonal axis; identifying workloads where adaptive quota actually triggers.

**Per-policy notes:** Individual observations grouped by LRU baseline, 2Q, CLOCK, two-hit/second-chance, and MRU-tail guard — including current calibration state and recommended next steps for each.


---

- finished: 2026-07-19T17:40:02+00:00
- duration: 61.08s
- status: ok
- input_tokens: 7
- output_tokens: 2407
- cache_read_tokens: 106134
- cache_write_tokens: 7690
- cost_usd: 0.100296
