# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-06-05T21:48:31+00:00

## timeline

### 21:48:54 thinking



### 21:49:01 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction *ordering*, not extra residency, is the core lever: gains come\n  from deciding which resident key dies first under a fixed capacity\n  bound. A \"protect on second access\" signal (segments, midpoint, ref\n  bit) is the foundational win that first beat pure LRU.\n- Enlarging the protected tier paid repeatedly across cycles, marching\n  80/20 → 90/10 → 95/5, each step a Winner. First-hit promotion plus a\n  generous protected tier is confirmed correct; reuse arrives fast enough\n  to land inside a narrow probationary window.\n- The least-regressive / confirmed-neutral change reliably \"wins\" a\n  cohort once the real levers are exhausted — several Winners are negative\n  confirmations rather than gains.\n- Among alternative skeletons, ARC's recency/frequency separation is the\n  first to show a genuine structural lift over the plain-LRU baseline.\n\n## Recurring failures\n\n- Loosening the promotion path always regresses: two-hit gates, wider\n  probationary windows, and over-trusting single post-eviction reuse all\n  lose keys that churn out before a second touch.\n- Non-recency signals consistently lose on this trace: front admission\n  windows (W-TinyLFU), LFU victim selection, and frequency at the\n  demotion point all underperform plain LRU inside the SLRU frame.\n- Ghost / reuse-distance re-admission is at best marginal and usually\n  redundant against a maximally-protected tier; correct placement (route\n  to probationary / protected-LRU end) only recovers a thin band.\n- Drain-floor protection in every form — static window, reprieve laps,\n  reactive minimum — fails; the binding constraint is the probationary\n  floor's *size*, and protecting the drain queue erodes it.\n- Adaptive geometry (floating the split on reuse feedback) drifts to a\n  worse partition than the static constant it replaced.\n- Transplanting SLRU tuning lessons onto LIRS is disconfirmed: lessons\n  inside one skeleton are not portable as parameter edits to another with\n  a different invariant structure.\n\n## Open questions\n\n- The persistent unresolved thread: alternative skeletons (LIRS in c0005,\n  ARC in c0008) report real hit-rate lifts yet never win the cohort. Is\n  the reported lift within noise, disqualified by a contract check, or\n  genuinely short of the incumbent? No clean head-to-head against the\n  95/5 SLRU incumbent has ever been run.\n- Is ARC's recency/frequency separation tunable (e.g. SLRU-style sizing\n  on its frequency tier) to exceed the incumbent, or is the static 95/5\n  optimum structurally unbeatable?\n- After many off-axis probes regressing, is any structural axis on this\n  trace left, or is the incumbent a true converged optimum?\n\n## Per-policy notes\n\n- SLRU: converged at 95/5. Geometry (static and adaptive), depth\n  stratification, drain-floor reprieves, and non-recency victim/admission\n  signals are all exhausted or harmful. Binding constraint is the\n  probationary floor needed to drain one-hit-wonders.\n- LIRS: home-turf for this trace family and the first off-axis probe to\n  report a gain, but every *tuning* attempt (HIR window, ghost diversion,\n  widened horizon, reactive floor) regressed. The clean untuned-parent\n  vs-incumbent verification remains never executed across c0005–c0008.\n- ARC: first skeleton to beat baseline structurally via two-sided\n  ghost-steered recency/frequency adaptation; not yet compared cleanly to\n  the incumbent. Live thread.\n- LFU / frequency-based: loses to recency on this reuse-heavy trace\n  wherever applied — admission, victim selection, or retention.\n"
}
```

### 21:49:15 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction *ordering*, not extra residency, is the core lever: gains come\n  from deciding which resident key dies first under a fixed capacity\n  bound. A \"protect on second access\" signal (segments, midpoint, ref\n  bit) is the foundational win that first beat pure LRU.\n- Enlarging the protected tier paid repeatedly across cycles, marching\n  80/20 → 90/10 → 95/5, each step a Winner. First-hit promotion plus a\n  generous protected tier is confirmed correct; reuse arrives fast enough\n  to land inside a narrow probationary window.\n- The least-regressive / confirmed-neutral change reliably \"wins\" a\n  cohort once the real levers are exhausted — several Winners are negative\n  confirmations rather than gains.\n- Among alternative skeletons, ARC's recency/frequency separation is the\n  first to show a genuine structural lift over the plain-LRU baseline.\n\n## Recurring failures\n\n- Loosening the promotion path always regresses: two-hit gates, wider\n  probationary windows, and over-trusting single post-eviction reuse all\n  lose keys that churn out before a second touch.\n- Non-recency signals consistently lose on this trace: front admission\n  windows (W-TinyLFU), LFU victim selection, and frequency at the\n  demotion point all underperform plain LRU inside the SLRU frame.\n- Ghost / reuse-distance re-admission is at best marginal and usually\n  redundant against a maximally-protected tier; correct placement (route\n  to probationary / protected-LRU end) only recovers a thin band.\n- Drain-floor protection in every form — static window, reprieve laps,\n  reactive minimum — fails; the binding constraint is the probationary\n  floor's *size*, and protecting the drain queue erodes it.\n- Adaptive geometry (floating the split on reuse feedback) drifts to a\n  worse partition than the static constant it replaced.\n- Transplanting SLRU tuning lessons onto LIRS is disconfirmed: lessons\n  inside one skeleton are not portable as parameter edits to another with\n  a different invariant structure.\n\n## Open questions\n\n- The persistent unresolved thread: alternative skeletons (LIRS in c0005,\n  ARC in c0008) report real hit-rate lifts yet never win the cohort. Is\n  the reported lift within noise, disqualified by a contract check, or\n  genuinely short of the incumbent? No clean head-to-head against the\n  95/5 SLRU incumbent has ever been run.\n- Is ARC's recency/frequency separation tunable (e.g. SLRU-style sizing\n  on its frequency tier) to exceed the incumbent, or is the static 95/5\n  optimum structurally unbeatable?\n- After many off-axis probes regressing, is any structural axis on this\n  trace left, or is the incumbent a true converged optimum?\n\n## Per-policy notes\n\n- SLRU: converged at 95/5. Geometry (static and adaptive), depth\n  stratification, drain-floor reprieves, and non-recency victim/admission\n  signals are all exhausted or harmful. Binding constraint is the\n  probationary floor needed to drain one-hit-wonders.\n- LIRS: home-turf for this trace family and the first off-axis probe to\n  report a gain, but every *tuning* attempt (HIR window, ghost diversion,\n  widened horizon, reactive floor) regressed. The clean untuned-parent\n  vs-incumbent verification remains never executed across c0005–c0008.\n- ARC: first skeleton to beat baseline structurally via two-sided\n  ghost-steered recency/frequency adaptation; not yet compared cleanly to\n  the incumbent. Live thread.\n- LFU / frequency-based: loses to recency on this reuse-heavy trace\n  wherever applied — admission, victim selection, or retention.\n"
}
```

### 21:49:25 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 8-cycle ledger, in the four required sections:

- **Recurring wins** — eviction *ordering* over residency, the "protect on second access" signal, the repeated payoff of enlarging the protected tier (80/20 → 90/10 → 95/5), and ARC as the first skeleton with a real structural lift.
- **Recurring failures** — loosened promotion, non-recency signals, ghost re-admission, drain-floor protection, adaptive geometry, and SLRU→LIRS lesson transfer.
- **Open questions** — the persistent never-run head-to-head of reported-gain skeletons (LIRS, ARC) vs. the 95/5 incumbent; ARC tunability; whether any structural axis remains.
- **Per-policy notes** — SLRU (converged at 95/5), LIRS (tuning fails, never cleanly verified), ARC (live thread), and frequency-based methods (lose to recency here).


---

- finished: 2026-06-05T21:49:25+00:00
- duration: 54.31s
- status: ok
- input_tokens: 5
- output_tokens: 3867
- cache_read_tokens: 70055
- cache_write_tokens: 14387
- cost_usd: 0.139083
