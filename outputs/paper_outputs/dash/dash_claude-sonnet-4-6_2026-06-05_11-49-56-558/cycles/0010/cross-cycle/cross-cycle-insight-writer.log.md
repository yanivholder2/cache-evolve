# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-06-06T02:13:29+00:00

## timeline

### 02:13:32 thinking

Let me analyze the ledger digest across all 10 cycles and write the cross-cycle-insights.md file. I need to find the output directory first.

### 02:13:33 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 02:13:33 tool: Glob

```json
{
  "pattern": "**/output*"
}
```

### 02:13:35 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/"
}
```

### 02:13:36 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve"
}
```

### 02:13:40 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/cross-cycle"
}
```

### 02:14:09 thinking

Now I know the output directory. I need to write the `cross-cycle-insights.md` file in `/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/cross-cycle/`.

Let me now synthesize all the patterns from 10 cycles into the four fixed sections.

**Recurring wins:**
1. Hot-tier topology expansion (increasing normal/stash ratio) — paid off from C1 through C8, ceiling confirmed at 7/8 (24/8 split, 28 normal buckets at C8/C9).
2. MRU insertion position — consistently dominant across C1–C6 and reinforced in C5; items inserted at MRU survive to be re-accessed.
3. Zero-access eviction guard (frequency guard) — introduced in C6, stacked successfully in C7. Protecting items with at least one in-tier hit from being evicted by never-accessed newcomers consistently pays off.
4. Second-chance stash overflow recovery (LRU/Front insertion) — introduced in C6, compounded in C7; inserting stash-evicted items at Front (LRU) of target tier proved additive with frequency guard.
5. Pre-insertion counter credit on confirmed-access paths — C9 innovation.

The counter credit before re-insertion on stash hits closed a real protection gap. On the flip side, I've tried frequency-based ranking multiple times across different versions, but the counter only works as a binary signal—zero versus any access—not as a ranking mechanism. Ghost caches and victim tracking showed minimal gains, and every attempt to delay MRU status through probationary zones or asymmetric insertion regressed performance.

Expanding the geometry past 7/8 makes the stash too shallow to recover from second-chance misses, and bypassing stash traversal altogether hurts hit rates since it's a critical quality filter. Timestamp-based promotion backfired because creation time contaminates the signal, letting cold items outrank hot ones.

I'm still wondering whether there's a structural loss I'm missing in demotion handling or insertion ordering, and whether soft-capping the counter to prevent staleness would actually matter at the 7/8 geometry, or if staleness isn't the real constraint.

On this Zipfian workload, position in the recency order is a much stronger reuse signal than frequency — ClassicLRU and MRU-insertion dominate because they encode that directly. The zero-access guard in ClassicLFUFreqProtect works because the binary threshold is the right abstraction; frequency counters are too noisy on this distribution to use as a ranking signal. Second-chance recovery helps when paired with the frequency guard, but LFU variants overall are structurally mismatched to the access pattern.

Front-insertion placement matters more than the decay formula itself — back/MRU insertion regressed, while front-insertion targets the right pipeline stage. Aging-LFU with symmetric decay actively harms performance by draining mildly-warm items, and even threshold-gated decay underperforms. TimestampLRU partially replicates the topology gains but can't surpass them and conflicts with frequency-guard eviction. Ghost caches and victim caches added only noise across configurations.

### 02:14:09 chat

Now I have enough to synthesize the 10-cycle ledger. Let me write the file.

### 02:14:38 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/dash/dash_claude-sonnet-4-6_2026-06-05_11-49-56-558/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Hot-tier topology expansion paid off across many cycles.** Shifting capacity from stash to normal buckets was the highest-leverage lever from C1 through C8, with each increment producing a real gain until the ceiling at a roughly seven-eighths normal fraction was confirmed in C9. Returns shrank monotonically with each step; the gain curve was concave throughout.\n- **MRU insertion is universally correct on workload-a.** Every variant that inserted at the MRU end of its target tier outperformed or held steady; every attempt to withhold MRU status (probationary positioning, textbook demotion cascades) regressed. Recency position is a stronger future-access signal than any guard or counter for this workload.\n- **Zero-access eviction guard (binary frequency threshold) is a clean win.** Protecting any item with at least one in-tier hit from being displaced by zero-access newcomers produced a clear improvement (C6) and compounded additively with second-chance stash recovery (C7). The binary threshold is the correct abstraction — the counter should never be promoted to a ranking signal.\n- **Second-chance stash recovery at Front/LRU is additive.** Unconditionally re-routing stash-evicted items to the LRU end of their target bucket stacks cleanly with the frequency guard because it targets a distinct pipeline stage (stash overflow vs. probing-bucket eviction). Front insertion correctly treats re-admitted items as probationers; Back/MRU insertion regressed (C8).\n- **Pre-insertion counter credit on confirmed-access paths closed a real protection gap.** Crediting `lfu_counter` before re-insertion at the stash-hit exit eliminated a window where a just-accessed item re-entered the target tier looking cold. Genuine but modest gain (C9).\n\n## Recurring failures\n\n- **`lfu_counter` as a ranking signal consistently hurts.** Frequency-ranked eviction, frequency-gated promotion, and frequency tiebreakers all regressed or were neutral (C1, C3, C10). The counter cannot distinguish newly-inserted items from genuinely cold ones; once the binary zero-vs-any threshold is crossed, recency order dominates.\n- **Aging-LFU in any form is contraindicated on workload-a.** Coarse symmetric decay evicted mildly-warm items with residual heat (C2); threshold-gated softer decay replaced that failure mode with a slow-ramp blindspot that still underperformed ClassicLRU (C3). Workload-a's distribution is recency-dominant and punishes frequency discrimination regardless of aggressiveness.\n- **Ghost/victim-cache re-admission produced noise across all configurations.** Tested in multiple cycles with different insertion positions and ghost-set sizes; the stash eviction boundary is not a meaningful hit-rate loss site on this workload. Retired after C5.\n- **Stash bypass or weakening always degraded quality.** Any variant that shortened stash traversal — front-door bypass, LFU-gated promotion delay — allowed cold or insufficiently-proven items to displace hot residents (C4). The stash is a load-bearing admission filter for this workload.\n- **Timestamp-based promotion is contaminated by insertion time.** Selecting promotion candidates by maximum `last_accessed` allows items whose creation timestamp postdates a hot item's last-access time to win promotion undeservedly. The resulting cold-item leak cancels or exceeds any eviction-selectivity gain (C6 v003, C7 v002).\n- **Geometry expansion past the seven-eighths ceiling regressed.** Reducing stash to fewer than four slots leaves insufficient headroom for cold-arrival bursts; second-chance recovery cannot compensate for the lost stash capacity (C9).\n\n## Open questions\n\n- **Soft-capping `lfu_counter` is the one counter-management axis never tested.** If long-lived hot items accumulate unboundedly high counts, newly warm items that should displace them cannot. This has been carried forward since C7 and remains unprobed; it could matter or be irrelevant at current geometry.\n- **Probing-bucket demotion handling has not been examined.** With topology, insertion position, eviction selectivity, and counter accounting all explored, structural asymmetry in how items are ordered after demotion from the normal tier into probing is the remaining uninvestigated pipeline stage.\n- **Would failed mechanisms compete on a different workload?** Ghost sets, frequency-ranked eviction, and admission gating all have plausible theory behind them. A scan-heavy or bursty cold-arrival trace might expose loss sites where these mechanisms pay off — the repeated finding that the stash is a quality gate for workload-a specifically suggests the workload is unusually stable.\n- **Is there a proportional or access-count-weighted re-entry position that extracts more from the insertion-position lever?** Full Back-insert is confirmed optimal, but a graduated placement based on accumulated hit evidence has not been tested.\n\n## Per-policy notes\n\n- **ClassicLRU / MRU-insertion baseline:** Dominant throughout; position encodes a stronger reuse signal than any counter on this Zipfian, recency-dominant workload. Any variant that degrades insertion position regreses.\n- **ClassicLRUFreqProtect (zero-access guard + second-chance + counter credit):** Current champion after C9. The binary zero-count threshold is the right abstraction. Do not extend the counter to ranking; do not change second-chance insertion from Front to Back.\n- **LFU variants (ranking):** Structurally incompatible with workload-a; recency order is more predictive than any frequency counter formulation tested. Should only be revisited with evidence from a different workload trace.\n- **Aging-LFU (coarse or threshold-gated decay):** Actively harmful regardless of decay rate; retired after C3.\n- **TimestampLRU:** Partially replicates gains already captured by topology and MRU insertion without surpassing them. Incompatible with frequency-guard eviction due to creation-time promotion contamination. Deprioritised.\n- **Ghost/victim cache:** Noise-level across every configuration and insertion position tested. Permanently retired.\n"
}
```

### 02:14:45 chat

Written `cross-cycle-insights.md` to the output directory. Here's a summary of what the file captures across all ten cycles:

**Recurring wins** — five patterns that reliably paid off:
1. Hot-tier topology expansion (C1–C8, concave gain curve, ceiling confirmed at ~7/8 split)
2. MRU insertion dominance — universally correct; withholding MRU status always regressed
3. Zero-access eviction guard — binary threshold is the right abstraction, not a ranking signal
4. Second-chance stash recovery at Front/LRU — additive with the frequency guard (different pipeline stage)
5. Pre-insertion counter credit on confirmed-access paths — closed a real but modest protection gap (C9)

**Recurring failures** — six approaches that kept not paying off:
- `lfu_counter` as a ranking signal (multiple cycles), aging-LFU in any form, ghost/victim caches, stash bypass/weakening, timestamp-based promotion (creation-time contamination), and geometry past the 7/8 ceiling

**Open questions** — four threads worth probing:
- Soft-capping `lfu_counter` to prevent staleness (mentioned since C7, never tested)
- Probing-bucket demotion ordering (last unexplored pipeline stage)
- Whether failed mechanisms would win on a scan-heavy or bursty-cold-arrival workload
- Proportional re-entry positioning based on accumulated hit evidence

**Per-policy notes** — observations grouped by eviction policy variant, including retirement status for ghost caches, aging-LFU, and TimestampLRU.


---

- finished: 2026-06-06T02:14:46+00:00
- duration: 76.76s
- status: ok
- input_tokens: 7
- output_tokens: 3505
- cache_read_tokens: 114836
- cache_write_tokens: 12653
- cost_usd: 0.14173
