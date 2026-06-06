# Cross-Cycle Insights

## Recurring wins

- Spending the baseline's existing long-term frequency signal more carefully
  (e.g. ranking a few LRU-tail candidates by frequency estimate before
  evicting) has produced the only clean hit-rate gain so far, with no
  regressions.

## Recurring failures

- Mechanisms that discard or bypass long-term frequency signal — aggressive
  sketch decay/aging and an unconditional admission window — have lost
  hit-rate. On this popularity-stable, low-churn workload they solve a problem
  the trace doesn't exhibit and only add noise.

## Open questions

- Are the inert mechanisms (sketch aging, admission window) failing due to
  parameterization (too-aggressive reset threshold, too-small window fraction)
  or because the workload genuinely lacks churn? Single-cycle evidence is
  mixed.
- Do frequency-aware victim-selection gains keep growing with a larger ranked
  sample, or plateau quickly because the trace has limited tail churn to
  exploit?

## Per-policy notes

- TinyLFU family: frequency-aware victim selection is the live winner; gains
  are slim, likely capped by how rarely the LRU tail hides a much-colder item
  than the head. Aging and the W-TinyLFU window are parked (mis-sized, not
  disproven) — revisit only as parameter sweeps or under a churnier workload,
  not as signal-discarding redesigns.
