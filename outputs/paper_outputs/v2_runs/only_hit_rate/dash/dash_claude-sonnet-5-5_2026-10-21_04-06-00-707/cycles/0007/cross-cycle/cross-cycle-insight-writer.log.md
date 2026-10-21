# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:34:17+00:00

## timeline

### 04:34:27 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Levers that use a real reuse or value signal pay off. Admission (second-sighting filter, cycle 3) and reuse-based victim choice (reference bit, cycle 6) were the only large movements.\n- Deciding what to admit or evict beats deciding where to place. Placement changes only won once (MRU on hit, cycle 1).\n- Gains shrink as state gets richer. A counter with aging on top of the reuse bit gave only a modest gain (cycle 7).\n- Structurally different levers have broken plateaus. Refinements of the same lever have not.\n\n## Recurring failures\n\n- Placement and position tweaks (insert, demotion, eligibility, two-tier insert, power-of-two buckets, wider probing) are flat or negative. Placement is saturated on workload-a.\n- Refining the admission filter (counters, doorkeeper, victim exemption, larger capacity, count-min, bypass when free slots exist) is flat or negative. The one-bit unconditional filter is a local optimum. Stacking admission mechanisms overlaps and does not add up.\n- Scan-order and second-chance tweaks on eviction have no effect.\n- \"Winner\" labels are often noise-level or only match the parent (cycles 2, 4, 5). They are not reliable evidence of improvement.\n- Several variants reproduced the parent exactly. The overlay edits may not have been on the live path, and authors could not read the sources, so mechanism claims are inferred.\n\n## Open questions\n\n- Do the gains hold on other workloads and cache sizes? All evidence comes from one trace at one size.\n- What do the wins cost in per-op-time and metadata? Only hit-rate has ever been measured, and the eviction scan and aging may be costly.\n- Is any hit-rate headroom left on this trace? Adaptive sizing, scan-resistance and other levers outside admission, placement and eviction state are untried.\n- Is the Winner rule relative to the cohort? Is the fitness vector missing dimensions?\n- Does victim selection that is confirmed live behave differently from the earlier no-op results?\n\n## Per-policy notes\n\n- Baseline with MRU-on-hit: a small early win. Further placement variants on top of it add nothing.\n- Second-sighting admission filter (cycle 3 v002): a solid gain. Treat it as fixed and keep it unconditional.\n- Reuse-bit eviction (cycle 6 v001): the largest gain so far. It needs no extra metadata in principle, since the flag lives in spare bits.\n- Two-bit aged counter eviction (cycle 7 v001): the current incumbent, a small win over the reuse bit. Its scan cost is unmeasured.\n- Count-min, TinyLFU-style and doorkeeper admission: little or no gain over the one-bit filter.\n- Power-of-two-choices and wider-probe structures: flat or slightly negative.\n"
}
```

### 04:34:30 chat

I wrote `cross-cycle-insights.md` to the output dir, built only from the seven cycle digests. It has the four required sections in order: Recurring wins, Recurring failures, Open questions, and Per-policy notes.

The main conclusions it records:
- **What worked:** deciding what to admit or evict has paid off, through the second-sighting admission filter and reuse-based victim choice. Placement tweaks have not.
- **What didn't:** refining the admission filter, scan-order tweaks, and other placement variants were flat or slightly negative. Several "Winner" labels are noise-level, and some variants matched their parent exactly, which may mean the edits never ran.
- **Open questions:** all evidence comes from one trace at one cache size, and only hit-rate has been measured, not per-op-time or metadata cost.
- **Incumbent:** the two-bit aged counter eviction from cycle 7 is the current incumbent, a small win over the reuse bit.

I included no numbers, since the digests give none.


---

- finished: 2026-10-21T04:34:31+00:00
- duration: 13.33s
- status: ok
- input_tokens: 4
- output_tokens: 1420
- cache_read_tokens: 38714
- cache_write_tokens: 10346
- cost_usd: 0.060106
