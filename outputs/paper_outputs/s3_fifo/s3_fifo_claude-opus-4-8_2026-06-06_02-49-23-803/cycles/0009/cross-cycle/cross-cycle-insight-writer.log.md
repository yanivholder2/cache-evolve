# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-06-06T06:17:02+00:00

## timeline

### 06:17:16 thinking



### 06:17:28 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Capacity reallocation over per-entry protection.** Moving room from the\n  small admission filter into main lifted hit-rate repeatedly (early cycles),\n  though the gains flattened toward a knee just past the narrowest split tried.\n- **Reward demonstrated reuse, never loosen the filter.** A higher freq\n  ceiling for already-hot keys gave small, real, split-orthogonal gains.\n- **Ghost length has a dominating knee at ~50% of main.** That point banked\n  the put-latency win *and* recovered hit-rate — a rare non-tradeoff move;\n  both wider and narrower windows were strictly worse on one axis.\n- **Eliminate scans structurally, don't reorder them.** O(1) hash indices\n  removed the resident-get walk (get path) and the ghost-membership walk (put\n  path), each lowering median *and* tail together — the strongest moves seen.\n- **Metadata discipline as a free dividend.** Retuning existing fields beat\n  adding state; once the index made the stored `key` redundant, deleting and\n  then relocating fields collapsed scored `Entry` size with zero hot-path cost.\n\n## Recurring failures\n\n- **Loosening admission/retention.** First-reuse promotion, non-zero ghost\n  re-admission, and freq=1 seeding for re-admitted keys all hurt hit-rate; the\n  baseline's strict freq=0 re-prove filter is load-bearing noise-screening.\n- **Widening the ghost queue.** Longer reuse-distance coverage readmitted\n  stale far-reuse keys, regressing hit-rate and growing metadata; this trace\n  rewards only narrow near reuse.\n- **Reordering a linear scan to make the median shallow.** Front-loading hot\n  keys, a dedicated hot segment, and bounded rotation loops all missed their\n  median targets and merely shifted cost into p99.\n- **Adding struct state for speculative benefit** (ghost sizing field, bounded\n  rotation) bought nothing and taxed metadata.\n\n## Open questions\n\n- The **unscored container-memory frontier** — separate ghost set vs.\n  folded-tombstone map — has been flagged for several cycles but never\n  directly contested; they make opposite tradeoffs for identical scored results.\n- Is the cohort now optimizing the **measurement** (padding, field location)\n  rather than real footprint? Does scoring still track the cost designs differ on?\n- With hit-rate, both hot-path scans, and scored `Entry` size all at floors,\n  is any scored axis left with genuine headroom?\n\n## Per-policy notes\n\n- **S3-FIFO on workload-a (all cycles).** Strict promotion/ghost thresholds\n  are protective, not over-aggressive — this workload is best served by\n  filtering tightly and spending freed capacity on main residency.\n- Reuse distance rewarded is **narrow but past the most-recent 25%** of\n  evictions; the recoverable near-reuse band sits at 25–50% of recent victims.\n- Latency tracked exactly one scan at a time (ghost membership on put, resident\n  walk on get); both are now O(1), so no hot-path linear walk remains.\n- Hit-rate has held **bit-identical for many cycles** — the policy is\n  hit-rate-bound at this capacity, and the remaining game is serving the same\n  hits more cheaply (latency, then scored/container memory).\n"
}
```

### 06:17:34 chat

Wrote `cross-cycle-insights.md` — a fresh projection of the 9-cycle ledger digest in the four fixed sections.

Key cross-cycle signals captured:
- **Recurring wins**: capacity reallocation over per-entry protection, rewarding proven reuse, the ~50% ghost-length knee, structural scan elimination (O(1) indices lowering median *and* tail), and metadata discipline.
- **Recurring failures**: loosening admission/retention, widening the ghost, reordering linear scans instead of eliminating them, and speculative struct-state additions.
- **Open questions**: the still-uncontested unscored container-memory frontier, whether the cohort is now optimizing measurement vs. real footprint, and whether any scored axis has headroom left.
- **Per-policy notes**: S3-FIFO on workload-a specifics — strict filtering is load-bearing, narrow near-reuse window, all hot-path scans now O(1), hit-rate floored for many cycles.


---

- finished: 2026-06-06T06:17:34+00:00
- duration: 31.77s
- status: ok
- input_tokens: 3
- output_tokens: 2229
- cache_read_tokens: 42914
- cache_write_tokens: 13036
- cost_usd: 0.102659
