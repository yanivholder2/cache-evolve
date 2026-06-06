# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-06-06T06:09:04+00:00

## timeline

### 06:09:16 thinking



### 06:09:24 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Reallocating capacity into the main queue (narrowing the small/main\n  split) was the first and most durable hit-rate lever — monotone for\n  several cycles until it flattened into diminishing returns.\n- Rewarding *demonstrated* reuse (taller freq ceiling) compounds with the\n  split and is orthogonal to admission — it deepens durability only for\n  keys that already earned it.\n- The ghost has a confirmed length knee at ~50% of main: it banks the\n  put-scan latency win and even recovers hit-rate, a rare non-tradeoff.\n- Eliminating hot-path linear scans structurally (O(1) hash index for get\n  in 0007, O(1) ghost membership for put in 0008) beats reordering them —\n  these moved long-frozen p50 floors and often p99 too.\n- Metadata discipline pays off later: the 0007 index made the per-entry\n  `key` field dead weight, deleted in 0008 to break a seven-cycle\n  metadata-size plateau.\n\n## Recurring failures\n\n- Loosening retention/admission filters (first-reuse promotion, non-zero\n  or freq=1 ghost re-admission) repeatedly hurt hit-rate: the baseline's\n  strict freq=0 re-proving is load-bearing noise filtering, not over-strict.\n- Widening the ghost queue never helps — this workload rewards near reuse,\n  not far reuse — and adding ghost-sizing struct state costs metadata.\n- Reordering a linear scan to make the median hit shallow (front-loading\n  hot keys, dedicated hot segment, bounded rotation loop) consistently\n  misses its target and merely shifts cost to the tail.\n- Stacking levers assumed additive often interferes (split × ghost),\n  though split × freq-ceiling proved genuinely orthogonal.\n\n## Open questions\n\n- Now that both hot-path scans are O(1), is there any latency or memory\n  lever left, or is the workload at a joint hit-rate / median-latency /\n  scored-size floor?\n- Which O(1)-ghost design wins once *unscored container memory* is weighed\n  — the duplicate-key HashSet or the tombstone-folded single map?\n- Do tombstone / dual-table designs accumulate stale entries that degrade\n  under longer or different inflow than tested?\n- Are more dead per-entry fields now deletable thanks to the hash index?\n\n## Per-policy notes\n\n- All cycles tune a single S3-FIFO variant; hit-rate has been bit-identical\n  for many cycles, treated as closed and capacity-bound at this size.\n- The policy's strictness (two-touch promotion, freq=0 ghost re-admission)\n  is protective on this near-reuse workload — retune existing constants\n  rather than relax filters or add state.\n- Latency tracks specific scans, not run-to-run jitter: per-op deltas are\n  noise unless a change alters an actual scan length or structure.\n- The frontier has migrated from hit-rate → ghost length/content →\n  structural scan elimination → unscored container/tombstone memory, which\n  is now the expected next binding constraint.\n"
}
```

### 06:09:29 chat

Wrote `cross-cycle-insights.md` — a fresh projection of the 8-cycle ledger across the four fixed sections:

- **Recurring wins:** split reallocation, orthogonal freq-ceiling durability, the ~50% ghost knee, structural scan elimination over reordering, and delayed metadata dividends.
- **Recurring failures:** loosening strict filters, widening the ghost, reordering linear scans to chase a shallow median, and assuming levers stack additively.
- **Open questions:** whether any lever remains past the joint floor, the unscored-memory reckoning between the two O(1)-ghost designs, tombstone staleness, and further dead-field deletions.
- **Per-policy notes:** S3-FIFO throughout, hit-rate closed/capacity-bound, strictness is load-bearing, latency tracks real scans, and the frontier's migration toward container/tombstone memory.


---

- finished: 2026-06-06T06:09:30+00:00
- duration: 25.93s
- status: ok
- input_tokens: 3
- output_tokens: 1572
- cache_read_tokens: 41873
- cache_write_tokens: 11367
- cost_usd: 0.085476
