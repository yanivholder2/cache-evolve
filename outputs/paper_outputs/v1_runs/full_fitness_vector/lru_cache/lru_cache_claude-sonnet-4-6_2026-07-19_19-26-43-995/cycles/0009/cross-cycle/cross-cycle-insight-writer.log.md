# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0009\cross-cycle
- allow_edits: True
- started: 2026-07-21T07:51:27+00:00

## timeline

### 07:51:33 thinking

Let me analyze the ledger digest across all 9 cycles and write a fresh `cross-cycle-insights.md` file. I need to find the output directory first.

### 07:51:33 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 07:51:33 tool: Glob

```json
{
  "pattern": "**/output*"
}
```

### 07:51:35 tool: Bash

```json
{
  "command": "ls /",
  "description": "List root directory"
}
```

### 07:51:37 tool: Glob

```json
{
  "pattern": "**/*.md"
}
```

### 07:51:38 tool: Read

```json
{
  "file_path": "/task.md"
}
```

### 07:51:41 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:/Users/yaniv/Projects/cache-evolve/outputs/lru_cache_2026-07-19_19-26-43-995/cycles/0009/cross-cycle"
}
```

### 07:52:17 thinking

The output directory is `C:/Users/yaniv/Projects/cache-evolve/outputs/lru_cache_2026-07-19_19-26-43-995/cycles/0009/cross-cycle`. I need to write `cross-cycle-insights.md` there.

Now let me synthesize the insights from all 9 cycles.

## Recurring wins

1. **Structural compaction of node metadata** — u32 sentinel links (cycle 0001), link-array split (cycle 0005) both reduced node size and improved cache-line density without adding per-op branches. These are the cleanest wins.
2. **87.5% protected quota (capacity/8 probationary)** — This quota held up across multiple clean-floor parents in cycles 0006-0009, consistently boosting hit-rate with no measurable latency cost.
3. **Simple structural changes beat complex algorithmic ones** — Every cycle where a policy mechanism added branching or indirection regressed. Every structural layout change that reduced node size or improved density either helped or stayed neutral.
4. **Two-queue policy with spare-bit encoding** — The protected-bit-in-prev encoding kept Node at 24 bytes while enabling full 2Q semantics.

Policy improvements that reuse existing fields at zero layout cost are reliable. The FNV-1a hasher showed genuine struct-size reduction and lower hash cost in cycle 0009, though I haven't yet tested it against the clean 87.5% baseline alone.

On the failure side, CLOCK and sweep-based eviction keep falling apart—hand-sweep cache-line pollution tanked cycle 0001, and Vec header bloat plus the same sweep cost killed cycle 0004. The amortized-relinking trade-off doesn't work when promotions and demotions are coupled. The SEEN_BIT two-hit admission filtering has failed repeatedly across cycles 0004-0006 in various forms because workload-a doesn't generate enough repeat accesses within the window to make it worthwhile.

Adaptive mechanisms like quota sizing and CountMin sketch graduation gates added overhead for logic that barely triggered—the Zipfian workload's hot set is too stable to benefit from phase-shift detection. Forced inlining and entry() API tricks in cycle 0002 backfired with instruction-cache displacement and extra borrow-checker work. Any quota above 87.5% consistently shows a real per-op-time regression across cycles 0007-0009, and three independent fixes (batching, FNV-1a, intermediate quotas) all failed to break that coupling. Demotion batching in cycle 0009 also didn't help.

The MRU-tail guard won on its own workload but failed when combined with 2Q in cycle 0003 because fire rate is too workload-dependent. I'm left with several open questions: whether FNV-1a on the clean 87.5% baseline could lower the per-op-time floor without a simultaneous quota change, whether node cache-line alignment and arena packing would show latency benefits at higher hit-rates, and what segmented approaches might offer.

I'm also considering whether lazy promotion in the LRU policy could achieve a similar protected ratio without the overhead of eager per-operation demotion, whether a frequency sketch could work as an ordering heuristic within the probationary queue rather than as a graduation gate, and what the minimum viable probationary pool size should be at 6.25% capacity. For the baseline LRU, node compaction brought it down to 16 bytes, but further improvements are constrained by how tight per-op latency already is.

The two-queue approach with protected and probationary splits remains the strongest direction—87.5% protected quota is the best confirmed threshold, and the protected-bit encoding keeps node size unchanged while enabling the semantics. Anything above that quota consistently regresses latency. CLOCK failed in both attempts due to sweep overhead and layout coupling between promotion and demotion rates. Two-hit admission filtering didn't work on this workload since there aren't enough repeat accesses within the window. The frequency sketch showed no discrimination as a graduation gate, though it might still be worth testing as an eviction ordering heuristic within the probationary queue itself.

### 07:52:17 chat

Now I have everything I need. Let me write the `cross-cycle-insights.md` file synthesizing all 9 cycles.

### 07:52:40 tool: Write

```json
{
  "file_path": "C:/Users/yaniv/Projects/cache-evolve/outputs/lru_cache_2026-07-19_19-26-43-995/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Node metadata compaction** reliably wins: u32 sentinel links (cycle 0001) and the link-array split (cycle 0005) each halved or quartered node size without adding per-op branches. Both landed as clean winners with zero latency regression.\n- **Two-queue policy via spare-bit encoding**: folding the queue-membership flag into the existing `prev` pointer kept Node layout unchanged while unlocking full 2Q semantics (cycle 0002). Reusing existing fields at zero layout cost is the most reliable pattern for policy additions.\n- **87.5% protected quota (capacity/8 probationary)**: confirmed cost-free on multiple independently-baselined parents across cycles 0006–0009. It consistently improves hit-rate without measurable latency regression and is the settled policy ceiling.\n- **Simple structural changes outperform complex algorithmic ones**: every cycle where a mechanism added branching or pointer indirection regressed; every pure layout change either won or did no harm. The workload's tight per-op budget is unforgiving to conditional logic.\n- **FNV-1a hasher**: confirmed genuine struct-size reduction and lower hash computation cost (cycle 0009); not yet tested in isolation at the clean baseline — a likely latency win still pending.\n\n## Recurring failures\n\n- **CLOCK / sweep-based eviction**: failed in cycle 0001 (hand-sweep cache-line pollution) and cycle 0004 (Vec header bloat plus same sweep cost at demotion). Amortised relinking is unfavorable when promotions and demotions are rate-coupled at steady state.\n- **SEEN_BIT / two-hit admission gate**: tested across cycles 0004, 0005, and 0006 in three distinct implementations (splice-on-first-hit, cost-free bit-OR, eviction-rescue). All failed for the same root cause: workload-a does not generate enough within-window repeat accesses for any two-hit requirement to be net beneficial.\n- **Quota above 87.5%**: cycles 0007, 0008, and 0009 all confirmed that any step beyond capacity/8 probationary carries a genuine per-op-time regression. Three independent remedies — intermediate quotas, demotion batching, FNV-1a offset — each failed to break the coupling.\n- **Adaptive mechanisms on a stationary workload**: adaptive quota sizing (cycle 0004) and CountMin sketch graduation gate (cycle 0007) both added overhead for logic that nearly never fired. The Zipfian hot set is stable enough to defeat any mechanism designed for phase shifts.\n- **Demotion batching**: defeated at high quotas because the tighter probationary window simultaneously raises graduation frequency, leaving net demotion burden unchanged or worse (cycle 0009).\n- **Forced inlining / `entry()` API tricks**: `#[inline(always)]` displaced hot-path code from instruction cache; the `entry()` API's borrow-checker constraint forced an extra pointer move on the dominant eviction path (cycle 0002). Both backfired.\n- **MRU-tail guard composition**: won as a standalone on its original workload (cycle 0002) but failed when composed with 2Q (cycle 0003) — fire rate too low to amortise the branch cost on a diffuse access pattern.\n\n## Open questions\n\n- **FNV-1a at the clean 87.5% baseline**: the hasher speedup has only ever been tested paired with a quota increase. Tested alone, it may lower the per-op-time floor and reopen a small window for future quota experiments.\n- **Node cache-line alignment / arena packing**: the 16-byte link-array node (cycle 0005) did not show a measurable latency benefit, but the gain is in principle real. At the current higher hit-rate from quota wins, does the layout dividend finally surface above the measurement floor?\n- **Lazy-promotion segmented LRU**: a fundamentally different policy structure that might achieve a similar effective protected ratio without eager per-operation demotion enforcement — the mechanism behind the confirmed latency cost.\n- **Frequency sketch as within-probationary eviction ordering**: using a sketch to evict estimated low-frequency items first (rather than strict FIFO) within the probationary queue remains untested; it avoids the graduation-gate failure mode that closed the sketch-as-filter branch.\n- **Minimum viable probationary pool diversity**: at 6.25% (capacity/16) the probationary budget may be too narrow to replenish the protected queue with sufficient item diversity; the reversal point has not been directly confirmed.\n\n## Per-policy notes\n\n- **LRU (baseline)**: structurally optimised to a 16-byte node via two successive compaction wins. Per-op latency is tight enough that any extra branch shows up immediately; further gains likely require layout or hash-lookup improvements, not behavioral changes.\n- **Two-queue (2Q) with protected/probationary split**: the dominant policy direction across all cycles. Best confirmed quota: 87.5% protected. Quota compression above that carries a real latency cost confirmed from three independent angles. The policy suits the stable Zipfian hot set well; adaptive variants add overhead for no benefit.\n- **CLOCK**: conclusively closed. Both the original sweep variant (cycle 0001) and the bitmap-backed deferred variant (cycle 0004) failed; the underlying trade-off — amortised relinking only pays when promotions vastly outnumber demotions — is not met at steady state.\n- **Two-hit / second-chance admission**: conclusively closed on this workload. Tested across four cycles in every plausible implementation; the workload simply does not produce the repeat-access density these mechanisms require.\n- **Frequency sketch**: as a graduation gate, found no discrimination signal on this workload (cycle 0007). As an eviction-ordering heuristic within probationary, untested and still open.\n"
}
```

### 07:52:49 chat

`cross-cycle-insights.md` written. Here's a brief summary of what was synthesized across the 9 cycles:

**Recurring wins** — Node metadata compaction (u32 sentinels → 16-byte link-array split), spare-bit 2Q encoding, the 87.5% protected quota as a confirmed cost-free ceiling, and the principle that pure structural changes consistently beat conditional algorithmic additions. FNV-1a is flagged as a probable win still needing a clean isolated test.

**Recurring failures** — CLOCK (twice, different implementations), SEEN_BIT/two-hit admission (three cycles, four variants), any quota above 87.5% (three independent fix attempts all failed to break the latency coupling), adaptive mechanisms on the stationary Zipfian workload, demotion batching, and forced-inlining micro-tricks.

**Open questions** — FNV-1a in isolation at the 87.5% baseline; whether the 16-byte node layout benefit finally surfaces above the measurement floor at current hit-rates; lazy-promotion segmented LRU as a policy redesign; and frequency sketch as within-probationary eviction ordering (not as a graduation gate).

**Per-policy notes** — Summarises the state of LRU, 2Q, CLOCK, two-hit admission, and frequency sketches as distinct policy families with their confirmed status on this workload.


---

- finished: 2026-07-21T07:52:50+00:00
- duration: 82.23s
- status: ok
- input_tokens: 8
- output_tokens: 3701
- cache_read_tokens: 148618
- cache_write_tokens: 12808
- cost_usd: 0.155658
