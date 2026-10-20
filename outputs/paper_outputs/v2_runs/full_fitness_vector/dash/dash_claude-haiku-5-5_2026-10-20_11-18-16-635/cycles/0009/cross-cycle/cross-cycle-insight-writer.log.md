# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T12:11:47+00:00

## timeline

### 12:11:56 thinking



### 12:12:02 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Protecting hot residents at eviction works better than protecting them at insert. The full-scan, lowest-counter victim choice has produced the only consistent hit-rate gains (cycles 0002, 0004, partly 0007).\n- Keeping the counter full-scan and non-decaying preserves that gain. Variants that shortened, sampled, or decayed the counter lost it (cycles 0005, 0007, 0009).\n- The deferred stash-to-target promotion keeps a hit-rate gain when its first-hit bookkeeping is cheap (cycles 0003, 0004). Its evidence is limited to workload-a.\n- Layout and replacement are separable. Packing or repacking records left hit rate and eviction order unchanged in every cycle that tested it (cycles 0004, 0005, 0006, 0007, 0008).\n\n## Recurring failures\n\n- Gating or delaying entry into the main table has lost hit rate. MRU insert displaced hot residents (cycle 0001), fullness-based insert placement cost hit rate (cycle 0002), blocking first-touch admission cost hit rate (cycle 0003), and ghost-ring re-admission cost a little hit rate (cycle 0006).\n- Cheapening the victim search has not made puts cheaper. Sampling, stash-only restriction, and early exit each left put latency at or above the parent's level (cycles 0005, 0007). Incremental minimum caching raised put and get tails (cycle 0006).\n- Counter-weight tweaks are inert or harmful. Doubling the increment and seeding new counters at 1 changed nothing measurable (cycles 0008, 0009). Sampling the increment on hits hurt hit rate (cycle 0009). Decaying counters hurt hit rate (cycle 0007).\n- Stash-side bookkeeping adds cost without a payoff. Occupancy checks and maintenance on the get path raised get and put latency (cycle 0002). Recency tie-breaking added eviction cost with no hit-rate change (cycle 0003).\n- Per-op latency claims have not held up in any cycle. Put and get movements repeatedly lack a mechanism in the edited code (cycles 0001 to 0009). Treat them as unconfirmed.\n- Winner labels have not tracked measured results. Variants with recorded regressions or no helped dimension were still labelled Winners (cycles 0001 to 0009).\n\n## Open questions\n\n- Does the full-scan lowest-counter gain hold on a second, non-workload-a trace? This check has been deferred since cycle 0004 and every hit-rate result still rests on workload-a.\n- Does the deferred stash-to-target promotion hold on a stash-dominant trace, as cycles 0003 and 0004 asked?\n- What drives put cost, given that shortening the scan did not help and the counter update is not the get bottleneck (cycle 0009)?\n- Does reordering the get lookup to skip the stash scan help or hurt stash-hit gets? The stash-hit tail has never been reported separately.\n- Where does the packed layout's counter live, and what does its update cost on the hit path? Packed-layout get and promotion regressions are still unexplained.\n- What is the true Winner criterion? The insights do not show which dimensions were weighted.\n\n## Per-policy notes\n\n- Frequency-aware victim selection (full-scan, saturating, non-decaying counter): the only policy with a repeated retention gain. Keep the full scan. Avoid sampled, two-choice, stash-only, decayed, or incrementally cached variants. Its put-path cost is real but its source is unidentified.\n- Insert and promotion placement (MRU insert, fullness-gated insert, admission delay, ghost ring): every variant that blocked, delayed, or displaced entries lost hit rate. Keep new keys able to enter the target bucket early.\n- Stash-to-target promotion (deferred): hit-rate neutral to positive when its first-hit state is cheap. Only tested on workload-a. Build on it only after a stash-dominant check.\n- Recency tie-breaking: no effect on hit rate. Drop.\n- Record layout (packed or repacked): does not change eviction but has not yielded a clean latency win. Any packing needs its hot-path decode and counter update measured before it is built on.\n"
}
```

### 12:12:05 chat

I wrote `cross-cycle-insights.md` in the cross-cycle output directory with the four required sections in order. I built it only from the inlined ledger digest and did not read the previous file.

The main signals across cycles 0001–0009:

- **Recurring wins:** The full-scan, non-decaying lowest-counter victim choice is the only repeated hit-rate gain. Keeping layout separate from replacement also held: packing or repacking records never changed eviction order. The deferred stash-to-target promotion gained hit rate in two cycles, but only on workload-a.
- **Recurring failures:** Blocking, delaying, or displacing entries at insert lost hit rate every time it was tested. Sampling, stash-only, decayed, or incrementally cached victim searches also lost ground, and none of them lowered put cost. Per-op latency claims and Winner labels have not tracked the measured results in any cycle.
- **Open questions:** The second-trace check has been deferred since cycle 0004, so every hit-rate result still rests on one workload. The stash-hit tail, put-side cost source, and packed-layout hot-path cost are also still unexplained.
- **Per-policy notes:** These group the observations by victim selection, insert and promotion placement, recency tie-breaking, and record layout.


---

- finished: 2026-10-20T12:12:05+00:00
- duration: 18.76s
- status: ok
- input_tokens: 4
- output_tokens: 3625
- cache_read_tokens: 42055
- cache_write_tokens: 15800
- cost_usd: 0.005393
