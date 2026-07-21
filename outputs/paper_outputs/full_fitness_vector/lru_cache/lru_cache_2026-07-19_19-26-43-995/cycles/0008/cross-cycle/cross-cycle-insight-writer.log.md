# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0008\cross-cycle
- allow_edits: True
- started: 2026-07-21T07:29:20+00:00

## timeline

### 07:29:24 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. I need to find the output directory first, then write the file with the four fixed sections.

Let me first find where the output directory is located.

### 07:29:24 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 07:29:24 tool: Glob

```json
{
  "pattern": "**/output*"
}
```

### 07:29:27 tool: Bash

```json
{
  "command": "ls /c/Users/yaniv/Projects/cache-evolve/ 2>/dev/null || ls C:/Users/yaniv/Projects/cache-evolve/ 2>/dev/null || echo \"not found\"",
  "description": "List cache-evolve project directory"
}
```

### 07:29:28 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:/Users/yaniv/Projects/cache-evolve"
}
```

### 07:29:33 thinking

Now I know the output directory structure. The allowed roots include:
1. `C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0008\cross-cycle`
2. `C:/Users/yaniv/Projects/cache-evolve/outputs/lru_cache_2026-07-19_19-26-43-995/cycles-ledger.jsonl`

So the output directory for the cross-cycle-insights.md file is:
`C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0008\cross-cycle`. I'll write the file there.

### 07:29:33 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0008\\cross-cycle"
}
```

### 07:30:13 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0008\\cross-cycle\\cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Structural compaction without added branching.** Reducing node size (u32 sentinel links in Cycle 0001, link-array split in Cycle 0005) consistently avoids latency regression and is the cleanest class of improvement. The wins came from denser cache-line utilization, not from any per-op logic change.\n- **Protected-quota enlargement improves hit-rate at low overhead — up to a point.** Each halving of the probationary pool (75% → 87.5% → 93.75% protected) delivered monotonically better hit-rate across Cycles 0003–0007. The 87.5% level is confirmed cost-free on both a standard and a link-array structural base; 93.75% carries a verified latency penalty.\n- **Policy changes that are genuinely zero-cost survive.** The MRU-tail guard (Cycle 0002) won by skipping no-op promotes at p50; the SEEN_BIT bit-OR (Cycle 0005) won by being structurally free. When added logic cannot be shown to cost nothing at the current measurement resolution, it consistently does not win.\n- **Simple structural changes outperform algorithmic complexity.** Across all eight cycles, the highest-confidence winners involved fewer, not more, operations per hot-path access.\n\n## Recurring failures\n\n- **CLOCK / deferred-relinking schemes.** Attempted in Cycle 0001 (hand-sweep pollution), Cycle 0004 (bitmap + Vec header bloat), and Cycle 0007 (demotion reordering). In every case the amortisation benefit is only available when promotions vastly outnumber demotions, a condition that cannot hold when the cache is full. Closed.\n- **Two-hit admission filters (SEEN_BIT gate).** Tested across Cycles 0004, 0005, and 0006 in three different implementations — splice-on-first-hit, cost-free bit-OR, and eviction-rescue variant. All failed because workload-a does not produce enough within-window repeat accesses for a stricter gate to be net beneficial. Closed on this workload.\n- **Frequency sketches as admission or eviction guides.** Cycle 0007's CountMin sketch and Cycle 0008's 2-candidate eviction scheme both found that a Zipfian moderate-frequency population already promotes correctly under immediate one-hit rules; adding sketch overhead changed nothing. Closed.\n- **Adaptive mechanisms on a stationary workload.** Adaptive quota sizing (Cycle 0004) never triggered because the Zipfian hot set is stable; the counters and branches paid cost for logic that never fired. Any mechanism designed for workload phase shifts is structurally wasteful here.\n- **Forced inlining and `entry()` API micro-optimization.** Cycle 0002/v003 showed that `#[inline(always)]` can worsen instruction-cache fit, and `entry()` hits a borrow-checker obstacle on the dominant eviction path. This class of optimization cannot be applied safely without workload-specific profiling.\n- **Demotion reordering at high quotas.** At 93.75% protected the overflow event barely fires, making placement of demoted items (MRU vs. LRU end of probationary) inert. Closed.\n\n## Open questions\n\n- **Where is the hit-rate/latency Pareto point between 87.5% and 93.75% quota?** An intermediate setting (e.g., capacity/12 or capacity/24) might recover part of the hit-rate gain while staying below the latency measurement floor. This is the most precisely scoped open experiment remaining on the quota axis.\n- **Can demotion overhead be batched or amortized?** The latency cost at 93.75% is driven by more frequent protected→probationary overflow. A mechanism that batches multiple demotion events could reduce per-op cost without changing effective quota.\n- **Does the link-array structural win compound with deeper quota?** The 16-byte node split (Cycle 0005) was cost-neutral at 87.5% quota and was tested at 93.75% (Cycle 0008/v003) where it confirmed the latency penalty — but the combination at an intermediate quota is untested.\n- **What axes remain after quota, admission, sketch, and layout are exhausted?** Possible unexplored directions: hash-map lookup overhead reduction, arena/slab alignment to reduce node-access cache-miss cost, or key-clone elision on a non-`Copy` key type (never validated on a heap-owning key).\n- **Is the minimum viable probationary pool size a hard floor?** At capacity/32 (~3%) the probationary queue may lack runway to replenish the protected queue with sufficient diversity. The reversal point has not yet been confirmed.\n\n## Per-policy notes\n\n- **LRU (baseline):** The reference policy is exact-order and well-suited to the Zipfian workload, which has enough reuse that eviction-quality precision pays off. All more complex policies have so far failed to improve on it without a structural co-change.\n- **Two-queue (2Q):** The protected-bit-in-prev encoding (Cycle 0002) kept Node at 24 bytes and enabled full 2Q semantics at no layout cost. The 87.5%/93.75% protected share improvements are cumulative on top of this structure. The policy is now the de facto baseline; further tuning is converging on a plateau.\n- **CLOCK:** Rejected twice (Cycles 0001 and 0004) on this workload. Cache-line pollution from hand sweeps and struct-size inflation from bitmap headers both outweigh the deferred-relink benefit. Not worth revisiting without a mechanism to suppress sweep-induced pollution.\n- **Two-hit / second-chance admission:** Every variant (SEEN_BIT splice, bit-OR, rescue, CountMin gate) was net negative on workload-a. The access pattern does not support a multi-hit precondition for promotion. Closed.\n- **Frequency-guided eviction within probationary:** Closed. At the small probationary pool sizes implied by high protected quotas, items are evicted before the frequency table accumulates distinguishable signal.\n"
}
```

### 07:30:21 chat

Written `cross-cycle-insights.md` to the Cycle 0008 cross-cycle output directory. The file synthesizes all eight cycles into the four fixed sections:

- **Recurring wins** — structural compaction, quota enlargement up to 87.5%, and genuinely zero-cost policy shortcuts all hold up across multiple cycles.
- **Recurring failures** — CLOCK/deferred-relinking (3×), two-hit admission filters (3×), frequency sketches (2×), adaptive mechanisms, and micro-optimization via forced inlining are all conclusively closed.
- **Open questions** — the sharpest remaining thread is finding the Pareto point between 87.5% and 93.75% quota; secondary threads cover batched demotion, link-array + intermediate quota composition, and unexplored axes (hash-probe cost, arena alignment, non-`Copy` key clones).
- **Per-policy notes** — LRU/2Q is the validated baseline; CLOCK and all multi-hit admission variants are closed on this workload; frequency-guided eviction is incompatible with deep-quota operation.


---

- finished: 2026-07-21T07:30:22+00:00
- duration: 62.08s
- status: ok
- input_tokens: 7
- output_tokens: 2533
- cache_read_tokens: 121904
- cache_write_tokens: 10804
- cost_usd: 0.121703
