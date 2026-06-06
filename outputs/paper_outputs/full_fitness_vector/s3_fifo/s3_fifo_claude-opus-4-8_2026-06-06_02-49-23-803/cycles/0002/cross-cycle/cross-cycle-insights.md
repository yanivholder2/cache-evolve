# Cross-Cycle Insights

## Recurring wins

- Reallocating capacity from the small admission filter into main is the
  single most reliable hit-rate lever (workload-a): it has paid off in
  every cycle that tried it, though gains are now shrinking.
- Rewarding *demonstrated* reuse — deepening durability only for keys that
  already proved hot (e.g. a taller frequency ceiling) — is a small but
  real, repeatable gain.
- Retuning existing fields beats adding state: every winning move so far
  re-used existing knobs rather than growing the control struct.

## Recurring failures

- Loosening the admission/promotion filter for weakly-touched items
  (first-reuse promotion, non-zero ghost re-admission) keeps lowering
  hit-rate. The baseline's strictness is load-bearing — it screens
  one-hit-wonders and short-lived entries.
- Widening reuse-distance detection (enlarging the ghost queue) keeps
  regressing: this trace rewards near reuse, not far reuse, and the
  ghost field also costs metadata for no benefit.
- Capacity knobs are not additive — stacking a second lever (ghost) onto
  the winning split has hurt rather than helped.

## Open questions

- Where is the split's knee? Each narrowing step under-screens
  one-hit-wonders more; the crossover where the filter finally fails has
  not been found — do not assume monotonicity.
- Are retention and capacity levers truly orthogonal? The freq-ceiling
  gain has not yet been tested combined with the narrowest split.

## Per-policy notes

- S3-FIFO (workload-a): hit-rate moves with the small/main split and with
  proven-reuse durability; it does not move with looser admission or
  longer reuse-distance detection. Per-op-time deltas read as run-to-run
  noise across cycles — no winning change altered the resident scan size —
  so treat timing as noise unless the hot path or resident set changes.
