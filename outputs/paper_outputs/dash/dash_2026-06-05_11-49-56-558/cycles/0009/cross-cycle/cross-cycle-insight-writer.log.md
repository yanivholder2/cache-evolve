# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-06-05T15:14:23+00:00

## timeline

### 15:14:26 thinking

Let me analyze the ledger digest across all 9 cycles and write a comprehensive `cross-cycle-insights.md` file. First, I need to find the output directory.

### 15:14:27 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 15:14:27 tool: Glob

```json
{
  "pattern": "output/**"
}
```

### 15:14:29 tool: Bash

```json
{
  "command": "ls -la",
  "description": "List files in current working directory"
}
```

### 15:14:32 tool: Read

```json
{
  "file_path": "/root/task.md",
  "limit": 30
}
```

### 15:14:34 tool: Bash

```json
{
  "command": "pwd",
  "description": "Show current working directory"
}
```

### 15:14:54 thinking

Now I know the output directory is `/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0009/cross-cycle`. I'll write `cross-cycle-insights.md` there.

Let me now synthesize insights from all 9 cycles:

**Recurring Wins:**
1. Hot-tier topology expansion (normal/stash ratio) — Cycles 1-3 showed this was the dominant lever; each expansion from 50/50 → 75/25 → (breakpoint at >28/4) gained hit-rate until 24/8 and then 28/4 was confirmed at the ceiling at 7/8


2. MRU insertion position — universally dominant; any attempt to withhold MRU status hurt performance
3. Zero-access eviction guard (frequency guard) — Cycle 6 breakthrough, protecting items with at least one in-tier hit from being evicted by newcomers
4. Second-chance stash recovery — Cycle 6, stacking with frequency guard in Cycle 7 confirmed additive gains
5. Stash-exit counter credit — Cycle 9 winner, closing protection gap for stash-demoted items re-entering the normal tier

**Recurring Failures:**
1. Frequency-based eviction (LFU, aging-...

Frequency-based eviction strategies consistently underperformed position-based recency on this workload, while ghost set re-admission proved near-neutral across configurations. Probationary positioning for new items and timestamp-based promotion both regressed performance, and geometry beyond the 7/8 normal-to-stash split showed diminishing returns. I'm still exploring whether soft-capping the counter prevents staleness in long-lived hot items, whether counter-management compounds with stash-exit credit, and whether a second workload trace would reveal benefits from frequency-based approaches. admission-control gains structurally blocked on workload-a
4. Whether probing demotion asymmetry is a remaining loss source

**Per-policy notes:**
- ClassicLRU / position-based recency: consistently strong on workload-a due to short reuse gaps and warm distribution
- ClassicLRUFreqProtect (frequency guard): the current champion baseline; zero-access threshold is correct for workload-a
- LFU variants: workload-contingent at best (workload-a is recency-dominant); co-resident decay is harmful
- Ghost set: structurally sound in theory but consistently near-zero on workload-a
- Second-chance stash recovery: real but smaller gain than eviction selectivity; LRU (front) insertion position is required
- Timestamp-based LRU: showed promise but couldn't match the position-based approach, with creation-time contamination being the core issue

### 15:14:54 chat

Now I have the output directory. I'll write the `cross-cycle-insights.md` based on the full ledger digest.

### 15:15:29 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Hot-tier topology expansion is the highest-leverage structural lever.** Moving the normal/stash split from equal halves toward a large-normal configuration (50/50 → 75/25 → 7/8) produced clear hit-rate gains across multiple cycles; each step confirmed the hot working set was still under-provisioned. The ceiling was located at roughly 7/8 (≈28 normal buckets); gains compound but diminish with each increment.\n- **MRU insertion position dominates universally on workload-a.** Every variant that placed newly-inserted or re-admitted items at the recency-hot (Back/MRU) end outperformed those that withheld MRU status. Short reuse gaps mean displaced items reliably get re-accessed before eviction, so probationary or LRU-entry positioning is consistently punishing.\n- **Zero-access eviction guard (frequency protect) produces a clear, durable win.** Preferring to evict items with no in-tier access over confirmed-reuse residents closed a real loss site at the probing boundary; the zero-access threshold is precisely right for this workload — single in-tier touches reliably predict membership in the warm working set.\n- **Second-chance stash recovery stacks cleanly with the frequency guard.** Unconditionally promoting stash-evicted items to the LRU end of their target bucket addresses a distinct pipeline stage (stash overflow) from the eviction guard (probing selectivity); the two gains are additive with no interference.\n- **Stash-exit counter credit closes a residual protection gap.** Crediting a `lfu_counter` access before re-inserting a stash-demoted item eliminates the window in which it is indistinguishable from a cold newcomer under frequency-protect eviction. Gain is modest but genuine, confirming that per-item accounting accuracy matters even at the exit boundary.\n\n## Recurring failures\n\n- **Frequency-based eviction (LFU and all aging variants) is structurally incompatible with workload-a.** Aggressive co-resident decay, threshold-gated decay, and standalone LFU all underperformed position-based recency. The workload's access distribution is warm and recency-dominant; item position encodes a stronger future-access signal than any frequency counter tested.\n- **Ghost set re-admission is near-zero across all configurations.** Ghost re-admission was tested at multiple positions (Front, Back) and yielded only noise-level movement every time. The workload does not cycle through the stash boundary with sufficient frequency for bypass tracking to pay off.\n- **LRU/Front insertion for newly-arriving or re-promoted items consistently regresses.** Probationary positioning — whether at ghost re-admission, demotion cascades, or probing entry — deprives items of MRU protection during their first in-tier access window, creating eviction blind spots rather than improving selectivity.\n- **Timestamp-based promotion is harmful when creation-time pollutes the ordering.** Recording timestamp on insertion allows cold items whose wall-clock creation is newer than a hot item's last access to win promotion, leaking cold items into the normal tier. This interaction destroyed the eviction-guard benefit when both were active.\n- **Geometry expansion beyond 7/8 is counterproductive.** Pushing to 15/16 left too few stash slots to buffer cold-arrival bursts; second-chance recovery could not compensate, and the marginal loss from stash shallowness overtook the marginal gain from wider normal capacity.\n- **Stash admission weakening always costs more than it saves.** Every attempt to bypass or gate the stash — front-door bypass for cold insertions, LFU-gated promotion — degraded hit-rate by allowing cold or slow-qualifying items to displace warm residents. The stash's role as an admission filter appears fundamental for this workload.\n\n## Open questions\n\n- **Soft-capping `lfu_counter` to prevent staleness.** Long-lived hot items may accumulate artificially high counts that crowd out newly-warm items under frequency-protect eviction. This axis has been flagged repeatedly but never tested; it is the most clearly identified untried lever.\n- **Whether stash-exit counter credit interacts with a soft cap.** If both are active, the credit sets a floor while the cap sets a ceiling on counter values; their interaction — cooperative or cancelling — is unknown.\n- **Whether timestamp promotion can be rehabilitated.** Recording a timestamp only on the first *hit* (not insertion) would eliminate creation-time contamination; the underlying concept may still be sound if that one failure mode is removed.\n- **Whether a second workload trace unlocks blocked axes.** Frequency discrimination, ghost re-admission, and probationary insertion have all been structurally ruled out for workload-a's Zipfian/recency-dominant pattern. A scan-heavy or bursty-cold trace might make these axes genuinely competitive.\n- **Intra-normal-tier eviction ordering at non-zero thresholds.** The frequency guard's zero-access cutoff is confirmed optimal; whether a graduated threshold (e.g., guard items with ≥2 hits once the normal tier is very wide) could extract further selectivity at 7/8 geometry has not been tested cleanly.\n\n## Per-policy notes\n\n- **ClassicLRU (pure position-based recency):** Solid baseline throughout; never catastrophically wrong on workload-a. Its recency ordering is the common substrate that all winning variants have preserved or strengthened.\n- **ClassicLRUFreqProtect (zero-access eviction guard):** Current champion policy. The zero-access threshold is precisely calibrated to workload-a; single in-tier touches are reliable reuse predictors and must remain protected. Stacks cleanly with second-chance recovery.\n- **Second-chance stash recovery:** Real, stable gain when insertion position is LRU (Front). Back/MRU insertion for second-chance items is definitively contraindicated — it grants unproven items immediate promotion candidacy.\n- **LFU and aging-LFU variants:** Only marginally competitive on frequency-skewed sub-traces; harmful on the broader workload-a distribution. Aggressive co-resident decay is the most harmful formulation. Should be reserved for workloads with confirmed frequency structure.\n- **Ghost-set re-admission:** Consistently noise-level; permanently retired for workload-a in all tested forms (conditional bypass, Front-insert, Back-insert).\n- **TimestampLRU:** Replicates some recency-ordering gains already captured by MRU insertion and topology; does not surpass the position-based approach, and is actively harmful when creation-time contaminates the ordering.\n- **Stash-exit counter credit:** Small but clean win; eliminates the zeroed-counter vulnerability for stash-demoted items re-entering the normal tier. Should be part of the standing baseline going forward.\n"
}
```

### 15:15:37 chat

`cross-cycle-insights.md` written. Here's a summary of what was synthesised across all nine cycles:

**Recurring wins** — Five durable patterns: hot-tier topology expansion (dominant for cycles 1–3, ceiling confirmed at 7/8); universal MRU insertion position; zero-access eviction guard (frequency protect, cycle 6 breakthrough); second-chance stash recovery stacking cleanly with the guard; and stash-exit counter credit closing a residual protection gap.

**Recurring failures** — Six consistently-failed axes: LFU and all aging variants (structurally incompatible with workload-a's recency-dominant distribution); ghost set (noise-level across every position and size configuration); LRU/Front insertion for newly-arriving items; timestamp-based promotion (creation-time contamination); geometry past 7/8 (stash too shallow); and any weakening of stash-as-admission-filter.

**Open questions** — Five threads: soft-capping `lfu_counter` to prevent staleness (most clearly identified untried lever); its interaction with stash-exit credit; rehabilitating timestamp promotion without creation-time pollution; testing a second workload trace to unlock structurally-blocked axes; and graduated frequency-threshold eviction at the current geometry.

**Per-policy notes** — Covers ClassicLRU, ClassicLRUFreqProtect, second-chance recovery, LFU variants, ghost-set, TimestampLRU, and stash-exit credit — each with a standing recommendation for future hypotheses-writer cycles.


---

- finished: 2026-06-05T15:15:37+00:00
- duration: 74.16s
- status: ok
- input_tokens: 7
- output_tokens: 3226
- cache_read_tokens: 108322
- cache_write_tokens: 10737
- cost_usd: 0.12768
