# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-06-05T21:26:20+00:00

## timeline

### 21:26:24 thinking



### 21:26:24 tool: Bash

```json
{
  "command": "ls; echo \"---PWD---\"; pwd; echo \"---existing---\"; find . -name 'cross-cycle-insights.md' 2>/dev/null | head",
  "description": "Locate output dir and existing insights file"
}
```

### 21:26:27 thinking



### 21:26:41 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-05_23-23-52-319/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction *ordering* beats extra residency: every gain came from\n  deciding which resident key dies first while holding the capacity\n  bound fixed.\n- \"Protect on second access\" is the core lever — distinguishing\n  one-hit-wonders from genuine reuse (segments, midpoint, reference bit)\n  is what consistently lifts hit-rate on this reuse-heavy trace.\n- Enlarging the protected tier paid off repeatedly (80/20 → 90/10 →\n  95/5), with each step a real but diminishing gain.\n- First-hit (eager) promotion is confirmed correct; loosening it always\n  cost hits.\n\n## Recurring failures\n\n- Non-recency / frequency signals keep losing inside the SLRU frame:\n  LFU victim selection, frequency-windowed eviction, and front admission\n  windows all regressed or went inert — reuse distances here exceed any\n  small window, so they evict keys before their second touch.\n- Ghost / reuse-distance re-admission is structurally redundant against a\n  heavily-protected tier: the warm-but-evicted band it can rescue is\n  vanishingly thin, and it costs extra scan/structure.\n- Loosening the parent (wider probationary, two-hit promotion gate)\n  consistently underperformed — reuse arrives fast, within the narrow\n  probationary window.\n- Transplanting SLRU tuning lessons onto LIRS failed: parameter wins are\n  not portable across skeletons with different invariant structures.\n\n## Open questions\n\n- Is the canonical LIRS rebuild's reported hit-rate lift real, and does\n  it beat the 95/5 SLRU incumbent on its home-turf trace? Multiple cycles\n  flagged this but it has never been cleanly re-run and verified against\n  the observable contract.\n- Has this trace's recency/promotion structure been fully consumed,\n  leaving only a fundamentally different policy (LIRS) as the live axis?\n- Is the LIRS parent at its own local optimum, or is there an untested\n  lever (LIR/HIR warm-up, non-resident bound) that actually moves it?\n\n## Per-policy notes\n\n- **SLRU**: the workhorse and current incumbent. Converged at 95/5;\n  geometry is exhausted — the binding constraint is the probationary\n  floor's *size* (needed to drain one-hit-wonders), not order within it.\n  Pushing to 97/3 regressed.\n- **LIRS**: the one off-axis mechanism to report a real gain on its\n  home-turf trace, but the lift remains unverified and never landed in\n  the Winners. SLRU-derived tunings (wider HIR window, weak-tier ghost\n  routing) regress it because its S-stack already does that filtering.\n- **LFU / frequency-based**: consistently harmful at near-full protected\n  tiers where freq values tie and mistrack recency.\n- **Pure LRU**: dominated baseline; all winners beat it via reuse-aware\n  ordering.\n"
}
```

### 21:26:45 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the six-cycle ledger, with the four fixed sections:

- **Recurring wins** — eviction ordering over residency, "protect on second access," enlarging the protected tier (with diminishing returns), eager first-hit promotion.
- **Recurring failures** — non-recency/frequency signals, ghost re-admission, loosening the parent, and cross-skeleton parameter transfer.
- **Open questions** — the still-unverified LIRS lift vs. the 95/5 incumbent, whether the recency axis is fully consumed, and whether LIRS has untested levers.
- **Per-policy notes** — grouped for SLRU (converged at 95/5), LIRS (real-but-unverified off-axis gain), LFU/frequency (consistently harmful), and pure LRU (dominated baseline).


---

- finished: 2026-06-05T21:26:46+00:00
- duration: 25.53s
- status: ok
- input_tokens: 5
- output_tokens: 1698
- cache_read_tokens: 63901
- cache_write_tokens: 10099
- cost_usd: 0.08706
