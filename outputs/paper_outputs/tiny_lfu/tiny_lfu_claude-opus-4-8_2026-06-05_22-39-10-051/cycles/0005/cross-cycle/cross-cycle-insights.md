## Recurring wins

- Spending the existing frequency signal more carefully on victim/promotion
  choice (rank a few tail candidates by estimate, gate SLRU promotion) is the
  only family of changes that has repeatedly cleared the noise floor — but
  always by a hair, never transformatively.
- Sharpening the estimator paid off once and decisively: widening the count-min
  sketch was the clearest single hit-rate lever, by buying a more accurate count
  rather than altering any policy mechanic.
- Structural recency protection (SLRU with a *large* protected tier) is
  directionally correct and currently holds the Winner substrate.

## Recurring failures

- Any mechanism that discards, decays, gates, or withholds long-term frequency
  signal loses: sketch aging, an unconditional admission window, the doorkeeper
  (refuted across multiple cycles), and deep victim sampling that trampled
  recency-warm keys before counts accumulated.
- Over-spending the signal backfires: widening the victim sample past its sweet
  spot plateaus then reverses; extra admission filtering only delays warming
  keys because the bare `incoming_freq > victim_freq` test already screens
  one-hit-wonders.
- Down-tuning structure hurts: shrinking the protected fraction regressed —
  the SLRU wants a larger protected tier, not a smaller one.
- "Wins" increasingly mean "did not regress" rather than lift, a sign the
  productive axes are mined out.

## Open questions

- Is hit-rate at this capacity now conclusively capped? Evidence from cycles
  0003–0005 strongly points to a workload ceiling rather than a tuning miss.
- Do the confirmed-positive refinements (victim sampling, frequency-gated
  promotion) compound when combined, or do they overlap on the same rare cases
  and net to nothing?
- Would the marginal structural levers pay off materially only under a churnier
  / more skewed workload — i.e. is the marginality intrinsic to this trace?
- With hit-rate near ceiling, is the more valuable search now efficiency/cost
  (e.g. narrower sketch + conservative update at equal accuracy, lower memory)?

## Per-policy notes

- Estimator (count-min sketch): fidelity axis is closed by direct test. Width
  and conservative update are redundant with each other, not additive; once
  cells separate the trace's distinct keys, further accuracy buys nothing.
- Admission gate: the simple `incoming_freq > victim_freq` test is already
  strong — it keeps the hot set resident and is the shared reason every
  structural refinement is marginal. Adding doorkeepers/windows on top only
  hurts.
- Victim selection: frequency-ranked sampling is real but marginal and has a
  narrow sweet spot; widening it reverses the gain.
- SLRU: independent of estimator accuracy, directionally positive, favors a
  large protected tier. Promotion gating and probationary victim sampling each
  nudge past parent by a hair because recency order already correlates with
  sketch frequency on this trace.
- Parked, not dead: sketch aging and admission window (likely mis-sized, only
  worth revisiting as parameter sweeps or under a churnier workload).
