# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-06-05T13:34:15+00:00

## timeline

### 13:34:18 thinking

The task is to write a `cross-cycle-insights.md` file based on the ledger digest provided. I need to find the output directory first, then write the file with the four fixed sections.

Let me first find the output directory.

### 13:34:19 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 13:34:19 tool: Glob

```json
{
  "pattern": "output*/**"
}
```

### 13:34:21 tool: Bash

```json
{
  "command": "ls -la",
  "description": "List current working directory"
}
```

### 13:34:25 tool: Read

```json
{
  "file_path": "/root/task.md"
}
```

### 13:34:27 tool: Bash

```json
{
  "command": "pwd",
  "description": "Show current working directory"
}
```

### 13:34:47 thinking

Good, I now know the output directory is `/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/cross-cycle`. I need to write `cross-cycle-insights.md` in this directory.

Now let me synthesize insights from the five cycles in the ledger:

**Recurring wins:**
1. Hot-tier expansion (topology) — dominant lever early; expanding normal bucket ratio consistently improved hit-rate across cycles 1 and 2
2. MRU insertion positioning — inserting/re-inserting at MRU tail consistently wins; appears universal for workload-a


3. Stash-as-admission-filter — preserving stash traversal is consistently beneficial; any attempt to bypass or weaken it regresses

**Recurring failures:**
1. Frequency-based eviction (LFU, aging-LFU) — tried multiple times across cycles 1-3 in various forms; consistently underperforms or causes regressions; workload-a appears recency-dominant
2. Ghost/victim cache re-admission — tried in cycles 4 and 5 in multiple configurations; consistently noise-level or slightly negative; stash boundary is not where meaningful hit-rate is lost
3. Stash bypass / admission weakening — consistently harmful (cycles 4+); weakening the stash quality filter always degrades performance
4. Non-MRU insertion positions — probationary, LRU-entry, demotion-cascade positioning all regress (cycle 5)

**Open questions:**
1. What happens at capacity stress (smaller total cache size) — would frequency-based or probationary approaches pay off?
2. Would frequency-based or scan-resistant schemes win on a different workload (scan-heavy, burst-cold, uniform)?
3. Are there genuinely new structural axes — adaptive segment sizing, size-aware eviction, TTL-based decay, cost-weighted eviction — that haven't been probed?
4. Is there a proportional or access-count-weighted re-entry positioning more nuanced than full MRU that could extract more from the position lever?

**Per-policy notes:**
- **LRU / MRU (position-based rec

**

LRU and MRU consistently dominate on this workload, with MRU insertion at every promotion and re-insertion point beating all alternatives. LFU and aging variants consistently underperform because aggressive per-hit aging evicts warm items too quickly, and the workload's reuse pattern is fundamentally recency-driven rather than frequency-driven. Segmented-LRU fails because demoting items from the hot tier still leaves them with residual heat, so preserving MRU privilege across segment boundaries is critical. Ghost sets show near-zero or negative impact across all configurations tested. The stash acts as a load-bearing admission filter, with the 24/8 split proving essential.

### 13:34:47 chat

Now I have the context I need. I'll write the `cross-cycle-insights.md` file synthesizing all five cycles.

### 13:35:08 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Hot-tier topology expansion** drove the largest hit-rate gains in cycles 1–2; redistributing capacity toward the normal (hot) tier consistently outperformed any policy change made at the same time. The gain curve peaked at 24 normal / 8 stash and has since been confirmed exhausted.\n- **MRU insertion positioning** is universally winning on workload-a; placing every item — whether newly inserted, promoted, or re-admitted — at the MRU end consistently preserves hits across all cycles tested. No non-MRU positioning variant has improved on this.\n- **Stash traversal as an admission filter** is load-bearing; every approach that weakened or bypassed it (direct insertion, ghost-bypass on cold items, LFU-gated promotion) regressed. Preserving full stash traversal is now a confirmed constraint, not a candidate for optimisation.\n- **Compounding additive axes** (topology + MRU re-entry together) reliably outperforms either axis alone, even when individual marginal gains become small.\n\n## Recurring failures\n\n- **Frequency-based eviction (LFU / aging-LFU)** has been retried in multiple forms across cycles 1–3 — standard LFU, aggressive per-hit aging, and threshold-gated decay — and consistently underperforms or regresses. Workload-a's access pattern is recency-dominant; frequency counters introduce blind spots during item ramp-up and drain mildly-warm residents that recency ordering would have kept.\n- **Ghost / victim-cache re-admission** has been tried in at least two cycles across multiple position configurations (MRU and Back-insert on re-admission) and consistently delivers noise-level or slightly negative movement. The stash eviction boundary is simply not where meaningful hit-rate is lost on this workload.\n- **Non-MRU insertion positions** (probationary zones, LRU-entry for new/re-admitted items, demotion-at-LRU asymmetry) regressed in every test in cycle 5; withholding MRU status increases eviction of items that still have residual access heat.\n- **Pushing hot-tier topology past the confirmed peak** (28/4 split) caused a regression in cycle 3; insufficient stash capacity prevents cold items from surviving long enough for re-entry.\n\n## Open questions\n\n- With topology, re-entry positioning, admission control, and ghost sets all explored or exhausted, what structurally new axes remain? Adaptive segment sizing, eviction cost weighting, and TTL-based decay have not been probed.\n- Would frequency-based or scan-resistant eviction become genuinely competitive under a different access pattern — scan-heavy, burst-cold, or uniform traces — where recency ordering is less dominant?\n- Does a capacity stress test (smaller total cache size) create conditions where probationary or frequency-gated schemes finally pay off, because marginal eviction quality matters more?\n- Is there a proportional or access-count-weighted re-entry position more nuanced than full MRU that could extract additional signal from the position lever?\n\n## Per-policy notes\n\n- **LRU / MRU (position-based recency):** Dominant on workload-a across all cycles. MRU insertion at every boundary — initial insertion, promotion, and re-admission — consistently preserves the most hits. The confirmed optimal geometry (24/8 split, MRU re-entry) forms the current compound baseline.\n- **LFU / aging-LFU (frequency-based):** Structurally incompatible with workload-a's access distribution. Aggressive decay drains warm items; threshold gating replaces one failure mode with another (slow counter ramp-up). Should not be revisited on this workload unless a different trace motivates it.\n- **Segmented-LRU (hot/cold asymmetry):** Textbook demotion-at-LRU is contraindicated; items displaced from the hot tier retain residual heat and need MRU privilege in the destination tier to survive to their next access. The classical asymmetric design assumes a sharper hot/cold divide than workload-a exhibits.\n- **Ghost sets:** Consistently noise-level on workload-a regardless of ghost set position or re-admission insertion point. Permanently retired as a lever for this workload.\n- **Stash (probing tier):** Confirmed role is admission filtering, not just capacity buffering. Optimal allocation is 8 slots (25% of total); both under-allocation (4 slots) and over-allocation (16+ slots) hurt. Any future variant must preserve full stash traversal for new cold insertions.\n"
}
```

### 13:35:14 chat

`cross-cycle-insights.md` written. Here's a summary of what was synthesised across the five cycles:

**Recurring wins** — Hot-tier topology expansion was the biggest early lever (peaked at 24/8 normal/stash); MRU positioning wins universally on workload-a; preserving full stash traversal is load-bearing; compounding additive axes reliably beats either alone.

**Recurring failures** — Frequency-based eviction (LFU, aging-LFU in any form) consistently underperforms on this recency-dominant workload; ghost sets are noise-level in every configuration tested; non-MRU insertion positions reliably hurt; pushing topology past the 24/8 peak regreses.

**Open questions** — Whether adaptive sizing, cost-weighted eviction, or TTL decay open new ground; whether frequency-aware schemes win on a scan-heavy or burst-cold trace; whether capacity stress changes the calculus.

**Per-policy notes** — LRU/MRU is dominant; LFU is retired for this workload; segmented-LRU's classical asymmetry is contraindicated; ghost sets permanently retired; stash slot count must stay near 8 (25% of total).


---

- finished: 2026-06-05T13:35:15+00:00
- duration: 59.38s
- status: ok
- input_tokens: 7
- output_tokens: 2613
- cache_read_tokens: 94917
- cache_write_tokens: 7515
- cost_usd: 0.099696
