## Recurring wins

- Eviction *ordering*, not extra residency, is the core lever: gains come
  from deciding which resident key dies first under a fixed capacity
  bound. A "protect on second access" signal (segments, midpoint, ref
  bit) is the foundational win that first beat pure LRU.
- Enlarging the protected tier paid repeatedly across cycles, marching
  80/20 → 90/10 → 95/5, each step a Winner. First-hit promotion plus a
  generous protected tier is confirmed correct; reuse arrives fast enough
  to land inside a narrow probationary window.
- The least-regressive / confirmed-neutral change reliably "wins" a
  cohort once the real levers are exhausted — several Winners are negative
  confirmations rather than gains.
- Among alternative skeletons, ARC's recency/frequency separation is the
  first to show a genuine structural lift over the plain-LRU baseline.

## Recurring failures

- Loosening the promotion path always regresses: two-hit gates, wider
  probationary windows, and over-trusting single post-eviction reuse all
  lose keys that churn out before a second touch.
- Non-recency signals consistently lose on this trace: front admission
  windows (W-TinyLFU), LFU victim selection, and frequency at the
  demotion point all underperform plain LRU inside the SLRU frame.
- Ghost / reuse-distance re-admission is at best marginal and usually
  redundant against a maximally-protected tier; correct placement (route
  to probationary / protected-LRU end) only recovers a thin band.
- Drain-floor protection in every form — static window, reprieve laps,
  reactive minimum — fails; the binding constraint is the probationary
  floor's *size*, and protecting the drain queue erodes it.
- Adaptive geometry (floating the split on reuse feedback) drifts to a
  worse partition than the static constant it replaced.
- Transplanting SLRU tuning lessons onto LIRS is disconfirmed: lessons
  inside one skeleton are not portable as parameter edits to another with
  a different invariant structure.

## Open questions

- The persistent unresolved thread: alternative skeletons (LIRS in c0005,
  ARC in c0008) report real hit-rate lifts yet never win the cohort. Is
  the reported lift within noise, disqualified by a contract check, or
  genuinely short of the incumbent? No clean head-to-head against the
  95/5 SLRU incumbent has ever been run.
- Is ARC's recency/frequency separation tunable (e.g. SLRU-style sizing
  on its frequency tier) to exceed the incumbent, or is the static 95/5
  optimum structurally unbeatable?
- After many off-axis probes regressing, is any structural axis on this
  trace left, or is the incumbent a true converged optimum?

## Per-policy notes

- SLRU: converged at 95/5. Geometry (static and adaptive), depth
  stratification, drain-floor reprieves, and non-recency victim/admission
  signals are all exhausted or harmful. Binding constraint is the
  probationary floor needed to drain one-hit-wonders.
- LIRS: home-turf for this trace family and the first off-axis probe to
  report a gain, but every *tuning* attempt (HIR window, ghost diversion,
  widened horizon, reactive floor) regressed. The clean untuned-parent
  vs-incumbent verification remains never executed across c0005–c0008.
- ARC: first skeleton to beat baseline structurally via two-sided
  ghost-steered recency/frequency adaptation; not yet compared cleanly to
  the incumbent. Live thread.
- LFU / frequency-based: loses to recency on this reuse-heavy trace
  wherever applied — admission, victim selection, or retention.
