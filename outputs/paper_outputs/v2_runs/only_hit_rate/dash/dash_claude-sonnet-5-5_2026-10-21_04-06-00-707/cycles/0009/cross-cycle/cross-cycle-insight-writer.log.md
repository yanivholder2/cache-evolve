# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:41:15+00:00

## timeline

### 04:41:25 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that decide *what* is kept beat changes that decide *where* it goes. The big moves were a second-sighting admission filter (cycle 3) and reuse-signal victim selection (cycle 6). The only early placement win was MRU on hit-promotion (cycle 1).\n- Using a real reuse signal (hit since insertion) for eviction broke a multi-cycle plateau. Richer state on top of it (aged counter) added only a small gain.\n- Keeping aging local to the evictor's scan beat global timer-driven aging.\n- Keeping the admission filter unconditional held up. It is not redundant with reuse-bit eviction.\n\n## Recurring failures\n\n- Placement and position tweaks (insert, demotion, eligibility, two-tier insert, power-of-two buckets, wider probing) were flat or negative. They are saturated on workload-a.\n- Refining or stacking admission machinery (counters, doorkeepers, bigger filters, count-min, victim exemptions, free-slot bypass) was flat or negative. Overlapping admission mechanisms do not add up.\n- Extra history or tiers (ghost table, protected bit, ghost filtering) gave noise-level gains at best, with unmeasured cost.\n- Scan order and second-chance fallback had no effect.\n- Since cycle 4, most Winner labels are noise-level or relative to the cohort only. They are weak evidence of improvement.\n- Several variants matched their parent exactly. Some overlay edits may have been dead code, and authors could not read the sources.\n\n## Open questions\n\n- Does any gain, especially reuse-based eviction and admission, hold on other workloads and cache sizes? All evidence is one trace and one size.\n- Is there any hit-rate headroom left on workload-a, or is every lever at the noise floor?\n- What are the metadata-size and per-op-time costs of the filters, counters, ghost state and scan work? Per-op-time and metadata-size have never been in the fitness vector, so small wins cannot be judged on cost.\n- Would a compact design (one-bit flag with bit-array filter) win on cost despite a marginal hit-rate gain?\n- Are there untried levers outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for other access patterns?\n\n## Per-policy notes\n\n- Baseline LRU-like promotion: MRU on hit-promotion helped. More promotion movement (eligibility, bidirectional swaps) did not.\n- Admission (second-sighting one-bit filter): this was the first real win, and it is the incumbent design. Capacity, aging, counters, victim conditioning and frequency sketches are all at or below noise.\n- Eviction (reuse bit, then aged 2-bit counter): this gave the largest and most recent real gain, and refinements since are diminishing. Ghost restore is marginal. Global-timer halving hurt. Scan order is irrelevant.\n- Bucket and probing structure: wider probing and power-of-two choices were no help. Dropping an incoming key when its bucket is all hot did almost nothing.\n- Compact one-bit reuse flag with bit-array filter: the hit-rate gain is marginal, and its cost savings are untested.\n"
}
```

### 04:41:29 chat

I wrote `cross-cycle-insights.md` in the output dir, with the four required sections in order. It's built only from the ledger digest for cycles 1–9, and I didn't open the previous insights file.

- **Recurring wins:** Admission and eviction levers paid off. The second-sighting filter (cycle 3) and reuse-signal victim selection (cycle 6) were the two big moves. Placement levers mostly didn't, apart from MRU on hit-promotion in cycle 1.
- **Recurring failures:** Placement tweaks, extra admission machinery, ghost and tier state, and scan-order changes were flat or negative. Most Winner labels since cycle 4 are noise-level.
- **Open questions:** Everything rests on one trace and one cache size. Per-op-time and metadata-size have never been measured. It's also unclear whether any headroom is left on workload-a.
- **Per-policy notes:** Grouped by baseline promotion, admission, eviction, bucket and probing structure, and the compact one-bit design.

I kept the numbers qualitative, as instructed.


---

- finished: 2026-10-21T04:41:29+00:00
- duration: 14.66s
- status: ok
- input_tokens: 4
- output_tokens: 1555
- cache_read_tokens: 40197
- cache_write_tokens: 11936
- cost_usd: 0.068113
