# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-06-10T20:38:26+00:00

## timeline

### 20:38:30 thinking



### 20:38:30 tool: Bash

```json
{
  "command": "pwd && ls",
  "description": "Show working directory and contents"
}
```

### 20:38:33 thinking



### 20:38:47 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Earned, durable protection beats raw recency.** Every hit-rate gain has\n  come from segmenting the bucket and shielding a proven working set from\n  one-hit-wonders and scan traffic — promote-on-(first/second)-hit plus a\n  load-bearing protected segment.\n- **Probation order is high-leverage.** Giving a fresh key a full lap to earn\n  promotion (oldest-first / FIFO probation) recovers rising keys that\n  newest-first churn discards — a clean hit-rate lift with no metadata change.\n- **Victim/admission logic is the productive surface.** The strongest wins\n  stayed intra-bucket (victim choice, probation order, segmentation); resident\n  set, routing, capacity invariants, and the policy enum were never touched.\n\n## Recurring failures\n\n- **Anything that erodes earned protection regresses.** Counter aging,\n  front/probationary admission composition, raising the promotion threshold,\n  capping/demoting the protected segment — all failed for the same reason on\n  this collision-bound trace: they make protection harder to earn or easier to\n  lose.\n- **Levers do not freely compose.** Stacking front admission onto\n  frequency-first eviction made fresh keys simultaneously oldest and\n  min-frequency, i.e. instant victims — orthogonal in isolation, conflicting\n  when combined.\n- **Approximating away granularity costs hit-rate.** Coarse one-bit (CLOCK)\n  recency underperforms full positional ranking in small buckets; geometry\n  coarsening is collision-bound with diminishing returns.\n\n## Open questions\n\n- Is the FIFO-probation + promote-on-first-hit scheme now at its hit-rate\n  ceiling, leaving only cost-side wins (banking reorder-free tiebreak savings)?\n- Does *looser* (larger, uncapped) protected sizing help, since tightening it\n  consistently hurts?\n- The O(bucket_size) linear victim/probation scan persists across all variants\n  but is largely unmeasured — what is its true per-op cost?\n- Is there any workload regime (shifting working set) where staleness would\n  dominate, or is this trace firmly collision-bound?\n\n## Per-policy notes\n\n- **Pure LRU (baseline):** victim/admission logic is the weak point; recency\n  alone under-protects the reusable working set against scans.\n- **Frequency-first / LFU-counter:** strong initial win by repurposing the\n  dormant counter as a live hit count; \"once-hot immortality\" is a feature\n  here, not a bug — aging it removed the protection that made it valuable.\n- **Segmented-LRU:** the durable winner. Explicit segmentation with\n  promote-on-reuse beat single-scalar frequency tuning. FIFO (oldest-first)\n  probation is the current best; one probationary hit is sufficient proof of\n  reuse; the protected cap/demotion branch is effectively dead code (segment\n  never saturates).\n- **CLOCK / reference-bit:** regresses — too coarse to replace positional\n  recency in small buckets.\n"
}
```

### 20:38:53 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 4-cycle ledger digest. The four fixed sections summarize the cross-cycle signal:

- **Recurring wins** — earned/durable protection over raw recency, probation-order leverage, and intra-bucket victim/admission as the productive surface.
- **Recurring failures** — every protection-eroding lever (aging, threshold-raising, capping, front-admission composition) regressing on this collision-bound trace, plus non-composing levers and granularity loss.
- **Open questions** — hit-rate ceiling, looser protected sizing, the unmeasured linear-scan cost, and whether any regime is staleness-dominated.
- **Per-policy notes** — grouped observations for pure LRU, frequency/LFU-counter, segmented-LRU (current winner), and CLOCK.


---

- finished: 2026-06-10T20:38:53+00:00
- duration: 26.90s
- status: ok
- input_tokens: 5
- output_tokens: 1674
- cache_read_tokens: 58278
- cache_write_tokens: 6699
- cost_usd: 0.070776
