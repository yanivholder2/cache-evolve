# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-06-05T21:35:25+00:00

## timeline

### 21:35:30 thinking



### 21:35:45 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction *ordering*, not extra residency, is the durable lever: every\n  early gain came from deciding which resident dies first under a fixed\n  capacity bound, anchored by a \"protect on second access\" signal.\n- Enlarging the SLRU protected tier paid repeatedly and monotonically\n  (80/20 to 90/10 to 95/5), each step landing the winner — until the\n  returns thinned to near-zero at the top of that range.\n- When an off-axis lever did help, it was placement at the *weak/LRU end*\n  of protection (routing ghost re-admissions to protected-LRU) rather\n  than straight-to-hot promotion.\n\n## Recurring failures\n\n- Loosening the promotion signal — wider probationary dwell, two-hit\n  promotion gates, depth/frequency-stratified protected tiers — always\n  regresses: reuse on this trace arrives fast and within a narrow window,\n  so anything that delays promotion or steals protected capacity loses\n  hits.\n- Non-recency admission signals (W-TinyLFU windows, LFU victim selection,\n  frequency at the demotion point) are inert-to-harmful: this trace is\n  not one-hit-wonder dominated, so scan-resistance machinery evicts\n  genuinely reused keys before their second touch.\n- Reuse-distance memory (ghosts) is structurally redundant once the\n  protected tier is large: hot keys are already retained, leaving only a\n  vanishingly thin warm-but-evicted band to rescue.\n- Adaptive geometry fails where static geometry succeeds: floating the\n  protected/probationary split on reuse feedback drifts to a worse\n  partition by eroding the probationary floor.\n- Cross-policy transfer is disconfirmed: SLRU tuning lessons applied as\n  parameter edits to LIRS regress, because the two skeletons have\n  different invariant structures.\n\n## Open questions\n\n- The long-standing unresolved thread (cycles 0005-0007): does an\n  *untuned* LIRS parent, cleanly re-run, actually beat the 95/5 SLRU\n  incumbent on its home-turf trace, and does it preserve the observable\n  contract? Every LIRS attempt so far either only *tuned* (and regressed)\n  or reported a lift that never landed in the Winners.\n- Is there any remaining structural axis on this trace, or is the 95/5\n  SLRU incumbent a true converged optimum? Three consecutive cycles of\n  off-axis probes have all regressed.\n\n## Per-policy notes\n\n- SLRU: the workhorse and repeated winner. Binding constraint is the\n  *size* of the probationary floor (drain capacity for one-hit-wonders),\n  not promotion timing or intra-segment order. Geometry is closed at\n  95/5 — both static (97/3 regressed) and adaptive resizing are\n  exhausted.\n- LIRS: the only genuinely off-axis mechanism to ever report a real gain\n  (home-turf trace), but the result is unverified and never won. Its own\n  S-membership/stack pruning already does the filtering SLRU bolts on;\n  transplanting SLRU knobs (HIR-window widening, weak-tier re-admission,\n  wider non-resident horizon) harms it because those signals are zero-sum\n  against its LIR budget.\n- LRU baselines: dominated from cycle 0001; pure recency is the wrong\n  axis and further pure-recency probes were dropped early.\n"
}
```

### 21:35:59 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction *ordering*, not extra residency, is the durable lever: every\n  early gain came from deciding which resident dies first under a fixed\n  capacity bound, anchored by a \"protect on second access\" signal.\n- Enlarging the SLRU protected tier paid repeatedly and monotonically\n  (80/20 to 90/10 to 95/5), each step landing the winner — until the\n  returns thinned to near-zero at the top of that range.\n- When an off-axis lever did help, it was placement at the *weak/LRU end*\n  of protection (routing ghost re-admissions to protected-LRU) rather\n  than straight-to-hot promotion.\n\n## Recurring failures\n\n- Loosening the promotion signal — wider probationary dwell, two-hit\n  promotion gates, depth/frequency-stratified protected tiers — always\n  regresses: reuse on this trace arrives fast and within a narrow window,\n  so anything that delays promotion or steals protected capacity loses\n  hits.\n- Non-recency admission signals (W-TinyLFU windows, LFU victim selection,\n  frequency at the demotion point) are inert-to-harmful: this trace is\n  not one-hit-wonder dominated, so scan-resistance machinery evicts\n  genuinely reused keys before their second touch.\n- Reuse-distance memory (ghosts) is structurally redundant once the\n  protected tier is large: hot keys are already retained, leaving only a\n  vanishingly thin warm-but-evicted band to rescue.\n- Adaptive geometry fails where static geometry succeeds: floating the\n  protected/probationary split on reuse feedback drifts to a worse\n  partition by eroding the probationary floor.\n- Cross-policy transfer is disconfirmed: SLRU tuning lessons applied as\n  parameter edits to LIRS regress, because the two skeletons have\n  different invariant structures.\n\n## Open questions\n\n- The long-standing unresolved thread (cycles 0005-0007): does an\n  *untuned* LIRS parent, cleanly re-run, actually beat the 95/5 SLRU\n  incumbent on its home-turf trace, and does it preserve the observable\n  contract? Every LIRS attempt so far either only *tuned* (and regressed)\n  or reported a lift that never landed in the Winners.\n- Is there any remaining structural axis on this trace, or is the 95/5\n  SLRU incumbent a true converged optimum? Three consecutive cycles of\n  off-axis probes have all regressed.\n\n## Per-policy notes\n\n- SLRU: the workhorse and repeated winner. Binding constraint is the\n  *size* of the probationary floor (drain capacity for one-hit-wonders),\n  not promotion timing or intra-segment order. Geometry is closed at\n  95/5 — both static (97/3 regressed) and adaptive resizing are\n  exhausted.\n- LIRS: the only genuinely off-axis mechanism to ever report a real gain\n  (home-turf trace), but the result is unverified and never won. Its own\n  S-membership/stack pruning already does the filtering SLRU bolts on;\n  transplanting SLRU knobs (HIR-window widening, weak-tier re-admission,\n  wider non-resident horizon) harms it because those signals are zero-sum\n  against its LIR budget.\n- LRU baselines: dominated from cycle 0001; pure recency is the wrong\n  axis and further pure-recency probes were dropped early.\n"
}
```

### 21:36:05 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 7-cycle ledger, with the four fixed sections:

- **Recurring wins** — eviction ordering over residency, the monotone payoff of enlarging the SLRU protected tier (80/20 → 95/5), and weak-end placement for re-admissions.
- **Recurring failures** — loosened promotion, non-recency admission signals, ghost re-admission, adaptive geometry, and cross-policy transfer all reliably regress, each with its structural reason.
- **Open questions** — the still-unresolved untuned-LIRS-vs-incumbent verification, and whether any structural axis remains.
- **Per-policy notes** — grouped for SLRU (converged at 95/5, probationary-floor bound), LIRS (only off-axis lever to report a real gain but never verified/won), and LRU (dominated early).


---

- finished: 2026-06-05T21:36:05+00:00
- duration: 40.44s
- status: ok
- input_tokens: 5
- output_tokens: 2999
- cache_read_tokens: 67132
- cache_write_tokens: 12439
- cost_usd: 0.117085
