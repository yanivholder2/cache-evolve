# Cycle 0007 Insight

## Cohort outcome

A cost-and-refresh cycle: all three variants branched off cycle-0005/v001
(compact layout + wide sketch) and probed whether the working base could
shed work or refresh its signal without losing hit-rate — trimming sketch
depth (v001), shrinking the victim sample (v002), and adding amortized
counter aging (v003). The two trim probes held hit-rate flat and each
banked one axis-local saving; the aging probe confirmed a long-standing
suspicion negatively.

## Patterns

- On the factor-16 sketch, depth and sample-depth are both past
  saturation for *accuracy*: dropping a min-row or cutting the victim
  sample left hit-rate essentially flat, confirming the wide sketch's
  estimate — not row count or walk length — is the binding constraint on
  admission and eviction quality. Their value is now purely cost-side:
  one buys footprint (a quarter of the sketch heap), the other shaves the
  put p99 tail.
- The predicted per-op-time *speedups* did not materialize. Removing one
  counter touch per access and two `estimate` calls per eviction both sat
  below the benchmark noise floor, so the summary per-op metric moved on
  run-to-run variance, sometimes the wrong way. At this width these knobs
  are size/tail levers, not hot-path-speed levers.
- Counter aging is settled as a net loss on this trace. Even tuned wide
  (interval = sketch width, single amortized sweep), halving decayed
  earned counts for a stable hot set faster than it refreshed contrast,
  dropping hit-rate while taxing every `increment`. The base's
  monotonically inflating counters were never the bottleneck here.

## Open questions

- With depth and sample-depth confirmed accuracy-saturated, is footprint
  (drop a row) worth banking permanently, or kept in reserve since it
  doesn't touch hit-rate?
- Is there any remaining lever that sharpens the wide sketch's estimate
  itself — the one thing shown to actually move hit-rate — that hasn't
  already been ruled flat or priced?

## Continuation

Close the aging thread for good: classic counter halving is a confirmed
loss on this stable-hot-set workload — do not revisit it. Treat sketch
depth and victim-sample depth as accuracy-saturated; tune them only for
footprint or tail, never expecting hit-rate or per-op speed. The frontier
is unchanged and narrowing: the only demonstrated hit-rate mover is
estimate sharpness (conservative update from Cycle 0006 remains the live
candidate), and the cheap free-lunch levers are exhausted. Next cycle
should test estimate-sharpening edits on the wide-sketch base rather than
turning any saturated size knob further.
