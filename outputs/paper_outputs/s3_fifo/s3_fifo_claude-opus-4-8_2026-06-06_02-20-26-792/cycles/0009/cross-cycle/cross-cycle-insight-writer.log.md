# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-06-06T03:16:11+00:00

## timeline

### 03:16:25 thinking



### 03:16:44 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Partition sizing was the dominant early lever: enlarging main (small/main from\n  baseline → 10/90 → ~6/94 → ~4/96) paid step after step until the knee, where\n  it froze. Most \"distinct\" retention knobs later collapsed back into this one.\n- Ordering discipline on main pays where protection does not: explicit\n  move-to-front recency beat position-blind FIFO, and the eviction-time\n  rotation (pop_back + push_front with per-survivor decay) proved load-bearing —\n  it refreshes survivor recency and targets cold-tail aging precisely.\n- Ghost *source quality* is a live axis: ghosting the strong main-evicted\n  population (keys that earned promotion) recovered a dropped signal, and\n  dropping weak records before strong ones under a fixed budget paid at the\n  margin.\n- Adaptivity finally beat static sizing: a source-steered (`from_main`) feedback\n  controller that nudges the split threshold, then momentum + a wider clamp,\n  each added a small real gain — the static ratio was slightly main-starved.\n- Standing principle: sharpening *which residents the eviction scan reaches*\n  pays; reordering beats re-admitting or warm-starting.\n\n## Recurring failures\n\n- Admitting more keys into main (looser promotion, frequency-ceiling deepening,\n  warm-started re-admission) crowds hotter residents — repeatedly regressive or\n  inert once main is large.\n- Ghost *depth* is falsified over both weak and strong populations: cooled keys\n  reaccess inside the original window or never; retained-past-bound entries\n  don't convert.\n- Ghost re-admission gating in every form (small-queue routing, throttling,\n  freshness/recency split) regressed — the direct-to-main path is correct; a\n  ghost hit is itself sufficient reuse evidence.\n- Frequency warm-start on re-admission regressed three separate times (weak and\n  strong populations alike) — protection only pays if reuse lands inside the one\n  protected pass, and too few keys do.\n- Victim sharpening (in-place decrement, windowed lowest-freq selection, tail\n  insertion of promoted/re-admitted keys) regressed — it discards the\n  rotation's recency refresh and demotes earned-front keys.\n- Global frequency aging over-decays (hot keys evicted before freq rebuilds);\n  weak-ghost removal is a severe regression (weak `from_main:false` records are\n  the dominant re-admission source); steering hysteresis and adaptive ghost\n  capacity both regress — the drift is monotone signal not noise, and ghost\n  capacity is the binding constraint that must stay pinned near `main_size`.\n\n## Open questions\n\n- Is the configuration now converged? Cycles 0003–0009 have produced only\n  sliver gains, one fully-negative cohort, and several falsified axes.\n- Adaptivity headroom appears capped by this trace's modest, near-monotone\n  drift — is there any controller refinement left, or a smarter steering signal?\n- With sizing, ordering, ghost depth/capacity/gating, and two runtime knobs all\n  explored, is there any mechanism not yet touched that could still pay?\n\n## Per-policy notes\n\n- All cycles tune one S3-FIFO-style policy (thin small admission queue, large\n  main, ghost history with a `from_main` source tag). The workload is a\n  LIRS-style trace with recency structure and modest, monotone phase drift.\n- The binding constraints are main retention (absorbed by the ~4/96 partition)\n  and raw ghost capacity (must stay pinned; depth cannot productively enlarge\n  it). Both weak and strong ghost populations are load-bearing — trim by source\n  priority, never delete a source.\n- The main queue's rotation-on-eviction and its front-insertion of\n  re-admitted/promoted keys are earned, not waste; do not disturb either.\n- Exhausted/falsified axes — do not revisit: static sizing, ghost depth, ghost\n  warm-start, weak-ghost removal, ghost re-admission gating, eviction ordering,\n  victim sharpening, insertion position, steering hysteresis, adaptive ghost\n  capacity. Adaptivity (split controller) is the most recently live axis but\n  looks near tuned-out.\n"
}
```

### 03:16:49 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 9-cycle ledger, using the four fixed sections. Key synthesis:

- **Recurring wins**: partition sizing (the dominant early lever, frozen at ~4/96), main-queue ordering discipline (move-to-front + load-bearing eviction rotation), ghost *source quality*, and adaptivity (source-steered split controller beating static sizing).
- **Recurring failures**: admitting/protecting weak-signal keys, ghost depth (falsified over both populations), all ghost re-admission gating, frequency warm-start (regressed 3×), victim sharpening, global aging, weak-ghost removal, hysteresis, and adaptive ghost capacity.
- **Open questions**: whether the config has converged, capped adaptivity headroom, and any untouched mechanism.
- **Per-policy notes**: grouped under the single S3-FIFO-style policy — binding constraints (main retention, pinned ghost capacity), load-bearing internals, and the consolidated exhausted/falsified axis list.


---

- finished: 2026-06-06T03:16:49+00:00
- duration: 38.34s
- status: ok
- input_tokens: 3
- output_tokens: 2934
- cache_read_tokens: 42626
- cache_write_tokens: 13423
- cost_usd: 0.114247
