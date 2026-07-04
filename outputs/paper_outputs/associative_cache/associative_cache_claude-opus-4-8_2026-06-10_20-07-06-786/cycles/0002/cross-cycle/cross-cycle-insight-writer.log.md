# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-06-10T20:23:14+00:00

## timeline

### 20:23:19 thinking



### 20:23:30 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Demanding proof of reuse before granting cache protection keeps winning:\n  frequency-first victim selection (C1) and segmented promote-on-second-hit\n  (C2) both beat their parents by shielding the reusable working set from\n  one-hit-wonders and scan traffic.\n- Repurposing the dormant `lfu_counter` as a live signal — a hit count (C1)\n  then a promote flag (C2) — has produced the cohort Winner two cycles\n  running.\n- High-leverage changes stay intra-bucket: victim/admission logic moves the\n  needle while resident-set structure, routing, capacity invariants, and the\n  policy enum / JNI mapping stay untouched.\n\n## Recurring failures\n\n- The orthogonal levers do not compose. Stacking probationary front\n  admission onto frequency-first eviction (C2 v001) makes a fresh key both\n  recency-oldest and min-frequency, so it is evicted immediately — a direct\n  conflict, not an additive gain.\n- Tuning a single frequency scalar underperforms explicit structure.\n  Counter aging (C2 v002) regressed by stripping the durable-count\n  protection that makes hot keys valuable.\n- Geometry coarsening / wider buckets (C1 v001) is collision-bound and gave\n  the smallest gain — diminishing returns, dropped.\n\n## Open questions\n\n- Can the O(bucket_size) linear victim/probationary scan be made O(1) (e.g.\n  a tracked segment boundary) without losing the hit-rate gain? Its per-op\n  cost has been structurally present but unmeasured every cycle.\n- Fitness vectors report only hit-rate; the cost side (per-op time, scan\n  cost) of each trade remains untracked.\n- Workload appears firmly capacity-collision-bound, not staleness-bound\n  (immortality helped, aging hurt). Is there any shifting-working-set regime\n  where staleness would dominate?\n- Is there a probationary/protected sizing ratio that beats the implicit\n  split, or does promote-on-second-hit already capture most of it?\n\n## Per-policy notes\n\n- LRU (baseline): pure-recency victim choice is the thing both winners\n  displace; weak against scan traffic and one-hit-wonders.\n- LFU / frequency-first: strong victim rule (C1 Winner); \"once-hot\n  immortality\" is a feature on this trace, so do not age counters. Still\n  pays a linear min-frequency scan.\n- Segmented-LRU (promote-on-second-hit): current strongest baseline (C2\n  Winner) and the best scan-resistant admission scheme so far; build cost\n  refinements on top of it.\n"
}
```

### 20:23:36 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the two-cycle ledger digest. Summary of what the long-memory file now captures:

- **Recurring wins** — "prove reuse before protecting" (frequency-first victim, then segmented promote-on-second-hit) keeps producing the Winner; repurposing `lfu_counter` as a live signal; gains stay intra-bucket.
- **Recurring failures** — the orthogonal levers conflict rather than compose (front admission + frequency-first), single-scalar tuning and counter aging regress, geometry coarsening is collision-bound.
- **Open questions** — the unmeasured O(bucket_size) scan cost, hit-rate-only fitness vectors, whether any regime is staleness-bound, and probationary/protected sizing.
- **Per-policy notes** — grouped observations for LRU, LFU/frequency-first, and Segmented-LRU (the current strongest baseline).


---

- finished: 2026-06-10T20:23:36+00:00
- duration: 22.07s
- status: ok
- input_tokens: 3
- output_tokens: 1435
- cache_read_tokens: 35192
- cache_write_tokens: 4533
- cost_usd: 0.050856
