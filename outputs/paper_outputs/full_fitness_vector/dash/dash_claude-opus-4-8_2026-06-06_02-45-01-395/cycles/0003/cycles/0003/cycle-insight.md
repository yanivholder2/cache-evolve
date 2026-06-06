# Cycle 0003 Insight

## Cohort outcome

All three variants forked from the cycle-0002 slim+MRU base and probed the
promotion-gate / hot-path axis: a wider positional window (v001), a
frequency-driven gate spending the free `lfu_counter` (v002), and a pure
branch-reorder latency play (v003). All three entered the Winners, but this
was a cohort of near-null results — every change landed within or below the
trace/measurement noise floor, with no clean compounding win among them.

## Patterns

- The promotion-gate axis has saturated on this trace. Widening the gate
  (v001) and switching it from positional to frequency (v002) each nudged
  hit-rate up only marginally, because MRU placement (`InsertPosition::Back`)
  already captures the warm mass either lever would protect — gate shape and
  MRU placement are substitutes, not additive. This directly answers cycle
  0002's open questions 1 and 2: protectable warmth is near-exhausted.
- Touching the hot `get` path to chase hit-rate carries real latency cost
  with little payoff: both the wider gate's promotion cascade (v001) and the
  per-hit counter read-modify-write (v002) raised p50 for a hit-rate gain
  indistinguishable from noise.
- Micro-latency reorders are below this harness's resolution. v003's branch
  reorder strictly removes a scan per hit yet moved latency the wrong way —
  and even `put`, which the change can't touch, regressed — confirming the
  saving is swamped by variance.

## Open questions

- With both placement and gate shape exhausted on hit-rate, is there any
  remaining lever on this trace that isn't noise-limited, or is hit-rate
  fully tapped?
- Since the harness can't resolve single-scan-per-hit savings, what
  granularity of structural change *is* large enough to measure?

## Continuation

Stop tuning the promotion gate and hot-path probe order — that axis is
saturated and its effects sit below the noise floor; further gate-shape or
reorder variants are not worth a slot. Pivot decisively to the
container/segment structure for the next real win (the size axis flagged in
cycle 0002), and size any future latency hypothesis to clear the measurement
noise floor rather than chasing per-hit micro-savings.
