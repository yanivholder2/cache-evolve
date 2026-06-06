## Recurring wins

- Spending the existing frequency signal *more carefully* keeps paying off:
  the only winners so far both improved the use/fidelity of long-term counts
  (a small frequency-ranked victim sample; a wider count-min sketch).
- The durable lever has trended upstream — from how the signal is *spent*
  (selection rule) toward how *accurately* it is measured (estimator
  fidelity). Buying a more accurate count beat tuning policy mechanics.

## Recurring failures

- Mechanisms that discard, decay, reset, or withhold long-term frequency
  signal consistently lose: aggressive sketch aging and an unconditional
  admission window (C1), a doorkeeper that clears fresh-key signal (C2).
- Spending the signal *more aggressively* backfires: deeper victim sampling
  plateaus then reverses, and gating mechanisms trample recency-warm or
  still-warming keys before their counts accumulate.
- Burst/shift-oriented machinery (aging, admission window) finds no problem
  to solve on this popularity-stable, low-churn trace and only adds noise.

## Open questions

- Does estimator fidelity (sketch width/depth) keep paying off when pushed
  further, or does it plateau once cells stop colliding with the distinct-key
  count?
- Are the parked mechanisms (aging, admission window, doorkeeper) inert from
  mis-parameterization or from a genuine lack of workload churn? Worth
  isolating an admission-only doorkeeper with a long clear window that never
  gates the main counters.
- Do the slim winning gains reflect a hard ceiling imposed by the trace's
  limited tail churn, or room that a churnier workload would reveal?

## Per-policy notes

- TinyLFU / count-min estimator: the dominant lever on this workload.
  Accuracy upstream (sketch fidelity) beats selection-rule tweaks; treat
  width/depth as the primary sweep axis until it plateaus.
- Victim selection: a *shallow* frequency-ranked tail sample is a small clean
  win; widening it is refuted. Keep near the original sweet spot.
- Admission control (window / doorkeeper): parked, not dead — likely
  mis-sized rather than wrong. Revisit only as constrained parameter sweeps,
  never resetting or gating the main frequency counters.
- Sketch aging/decay: parked; assumes shifting popularity this trace does not
  exhibit.
