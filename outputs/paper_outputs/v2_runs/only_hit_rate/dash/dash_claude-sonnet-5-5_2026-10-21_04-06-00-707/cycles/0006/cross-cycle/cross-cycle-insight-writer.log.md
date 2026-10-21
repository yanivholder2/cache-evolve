# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:31:05+00:00

## timeline

### 04:31:15 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Levers that change *which* entries stay resident beat levers that change *where* they sit. The two clear gains were the second-sighting admission filter (cycle 3) and reuse-bit victim selection (cycle 6).\n- Promotion position (MRU on hit) was the only placement change that helped (cycle 1). It did not stack with later placement tweaks.\n- Using a real reuse signal (hit since insertion) for eviction broke the plateau that admission and placement tuning had reached.\n- Simple, near-zero-metadata mechanisms have outperformed more elaborate ones.\n\n## Recurring failures\n\n- Placement-position tweaks (insert, demotion, eligibility, two-tier insert) have been flat or noise-level since cycle 2. They are saturated on workload-a.\n- Refining or stacking admission machinery (counters, doorkeepers, victim exemption, larger filters, count-min gates) lost or gained only noise. The one-bit filter looks like a local optimum, and overlapping admission mechanisms do not add up.\n- Extra movement (bidirectional swaps, wider probe neighbourhoods, power-of-two bucket choice) gave no gain and sometimes cost hit-rate through cascades or by keeping cold entries resident.\n- Several results matched the parent's hit-rate exactly, which suggests dead or non-live overlay edits (mostly victim selection in cycles 2-3).\n- Winner labels were sometimes assigned to noise-level or sub-parent variants (cycles 4, 5), so the winner rule is unreliable without cost dimensions.\n\n## Open questions\n\n- Do the admission and reuse-bit eviction gains generalize to other workloads and cache sizes? Every result so far comes from workload-a.\n- Can a richer reuse signal do better than one bit (multi-bit aging, CLOCK-style second chance, scan resistance)? Does it stack with the admission filter?\n- What are the per-op-time and metadata costs of the eviction scan, the filter state and the extra probes? None have been measured, because only hit-rate has been reported.\n- Are the overlay edits reaching the executed path? Authors cannot read the sources, so mechanism claims remain inferred.\n- Is the winner-selection rule relative to the cohort, or is the fitness vector missing dimensions?\n\n## Per-policy notes\n\n- Baseline/promotion (cycle 1): MRU insertion on hit helped. Added promotion movement did not.\n- Admission (second-sighting one-bit filter, cycle 3): the incumbent for admission. Variants (counter, doorkeeper, victim-conditional, 4x capacity, count-min) were all flat or worse.\n- Eviction (reuse-bit, first unreferenced slot, cycle 6): the current best and incumbent. Furthest-from-MRU scan (cycle 3) and cold-end demotion (cycle 2) showed no effect, possibly because they were not live.\n- Bucket/probing structure: wider probe and power-of-two choices gave no benefit. Treat the structure as low-yield.\n- Next direction: branch from the reuse-bit eviction incumbent, keep the admission filter, and add cost metrics to the fitness vector.\n"
}
```

### 04:31:18 chat

I wrote `cross-cycle-insights.md` in the cycle 0006 cross-cycle output directory. It has the four fixed sections in the required order. It is built only from the six-cycle ledger digest, without reading the earlier insights file.

The main takeaways it records:
- **Wins:** Changing which entries stay resident has paid off more than changing where they sit. The two clear gains were the second-sighting admission filter (cycle 3) and the reuse-bit victim selection (cycle 6).
- **Failures:** Placement tweaks, extra admission machinery and extra movement are flat or at noise level on workload-a.
- **Open questions:** Whether the gains hold on other workloads and cache sizes, whether a multi-bit reuse signal beats one bit, and what the costs are. Per-op-time and metadata are still unmeasured, and some earlier edits may never have run.
- **Per-policy notes:** Reuse-bit eviction is the current incumbent, with the one-bit admission filter kept alongside it.


---

- finished: 2026-10-21T04:31:19+00:00
- duration: 14.25s
- status: ok
- input_tokens: 4
- output_tokens: 1533
- cache_read_tokens: 38007
- cache_write_tokens: 9748
- cost_usd: 0.058703
