# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T12:18:09+00:00

## timeline

### 12:18:16 thinking



### 12:18:20 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Deferred stash-to-target promotion keeps a hit-rate gain across cycles 0003 and 0004, as long as the first-hit state stays cheap on stash hits.\n- Full-scan lowest-counter victim choice is the only victim policy that has produced a hit-rate gain. Its base is a saturating counter that does not decay.\n- Retention depends on hits accumulated after insertion. Changes to the starting value or the per-hit increment were flat, and sampling the increment cost hit rate.\n- Layout and replacement are separable. Record repacking and packed layouts moved metadata size without changing eviction order, so layout work can be judged on footprint alone.\n\n## Recurring failures\n\n- Narrowing the victim search lost the gain. Two-choice sampling and stash-only restriction both fell below the full-scan result, so the retention effect needs the true lowest-counter resident.\n- Keeping the minimum up to date costs more than the scan it replaces. The incremental victim cache raised put and get tails.\n- Counter decay lowered hit rate, and a decay pass was the worst put result in the cohort.\n- Protecting residents at insert time has not helped. Gating promotion position on fullness cost hit rate, and blocking first-touch admission also lost hit rate.\n- Ghost-ring re-admission and per-slot tag filters lost hit rate or added put-path bookkeeping without a gain.\n- Cutting the victim scan short or sampling it did not reduce put latency in three cohorts in a row. Put cost is not explained by scan length.\n- Recency tie-breaks, residency-class tie-breaks, and overwrite counting moved nothing that helps.\n\n## Open questions\n\n- The second-trace check requested since cycle 0004 has never been run. Every hit-rate result so far is workload-a only, so no directional claim generalises yet.\n- The get-path regression appears across many variants and may be shared. Re-measure the parent before judging any further get or put result.\n- The stash-hit tail is not split out in the fitness vector, so the effect of reordering the `get` lookup is unknown on that path.\n- Why are variants labelled Winners when they record regressions and no helped dimension? The Winner basis is not visible in any insight.\n- Can the packed record keep its footprint saving while the hit-path counter stays unpacked?\n- The overlay source has not been inspected in any cycle, so per-op attributions remain inferred and should not be credited.\n\n## Per-policy notes\n\n- Full-scan lowest-counter (saturating, non-decaying): the working base. Keep the full scan and avoid sampling, caching the minimum, or decaying counts.\n- Promotion and admission: the deferred stash-to-target promotion is the candidate to build on. Admission delays lost hit rate on workload-a, and insert-position changes trade hit rate for protection.\n- Tie-breaking among equal counters: age and residency-class tie-breaks were marginal or flat, so this is a low-priority lever.\n- Ghost ring and stash-only variants: not carried forward.\n"
}
```

### 12:18:23 chat

I wrote `cross-cycle-insights.md` to the cycle 0010 cross-cycle output directory. It has the four fixed sections in order, and I did not read the previous version of the file.

The main signals across cycles 0001–0010:

- **Recurring win:** the deferred stash-to-target promotion, which kept a hit-rate gain in cycles 0003 and 0004. The full-scan, non-decaying lowest-counter victim choice is the only victim policy that has produced a hit-rate gain.
- **Recurring failures:** narrowing the victim search (sampling, stash-only) lost that gain. Keeping the minimum incrementally, counter decay, insert-time gating, ghost-ring re-admission and per-slot tags did not pay off. Shortening the scan has not lowered put latency in three cohorts in a row.
- **Biggest gap:** every hit-rate result comes from workload-a only. The second-trace check has been deferred since cycle 0004.
- **Measurement caveat:** Winner labels are not explained by the recorded regressions. The get regression appears across several variants, so the parent should be re-measured before any latency claim is credited.


---

- finished: 2026-10-20T12:18:24+00:00
- duration: 14.72s
- status: ok
- input_tokens: 4
- output_tokens: 2540
- cache_read_tokens: 43081
- cache_write_tokens: 15784
- cost_usd: 0.004858
