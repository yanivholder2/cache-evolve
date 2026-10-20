## Recurring wins

- Eviction *ordering* beats extra residency: every gain came from
  deciding which resident key dies first while holding the capacity
  bound fixed.
- "Protect on second access" is the core lever — distinguishing
  one-hit-wonders from genuine reuse (segments, midpoint, reference bit)
  is what consistently lifts hit-rate on this reuse-heavy trace.
- Enlarging the protected tier paid off repeatedly (80/20 → 90/10 →
  95/5), with each step a real but diminishing gain.
- First-hit (eager) promotion is confirmed correct; loosening it always
  cost hits.

## Recurring failures

- Non-recency / frequency signals keep losing inside the SLRU frame:
  LFU victim selection, frequency-windowed eviction, and front admission
  windows all regressed or went inert — reuse distances here exceed any
  small window, so they evict keys before their second touch.
- Ghost / reuse-distance re-admission is structurally redundant against a
  heavily-protected tier: the warm-but-evicted band it can rescue is
  vanishingly thin, and it costs extra scan/structure.
- Loosening the parent (wider probationary, two-hit promotion gate)
  consistently underperformed — reuse arrives fast, within the narrow
  probationary window.
- Transplanting SLRU tuning lessons onto LIRS failed: parameter wins are
  not portable across skeletons with different invariant structures.

## Open questions

- Is the canonical LIRS rebuild's reported hit-rate lift real, and does
  it beat the 95/5 SLRU incumbent on its home-turf trace? Multiple cycles
  flagged this but it has never been cleanly re-run and verified against
  the observable contract.
- Has this trace's recency/promotion structure been fully consumed,
  leaving only a fundamentally different policy (LIRS) as the live axis?
- Is the LIRS parent at its own local optimum, or is there an untested
  lever (LIR/HIR warm-up, non-resident bound) that actually moves it?

## Per-policy notes

- **SLRU**: the workhorse and current incumbent. Converged at 95/5;
  geometry is exhausted — the binding constraint is the probationary
  floor's *size* (needed to drain one-hit-wonders), not order within it.
  Pushing to 97/3 regressed.
- **LIRS**: the one off-axis mechanism to report a real gain on its
  home-turf trace, but the lift remains unverified and never landed in
  the Winners. SLRU-derived tunings (wider HIR window, weak-tier ghost
  routing) regress it because its S-stack already does that filtering.
- **LFU / frequency-based**: consistently harmful at near-full protected
  tiers where freq values tie and mistrack recency.
- **Pure LRU**: dominated baseline; all winners beat it via reuse-aware
  ordering.
