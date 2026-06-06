# Cycle 0001 Insight

## Cohort outcome

All three variants were independent, single-lever tweaks to the CountMinSketch
off a shared baseline: frequency aging (v001), single-hash double-hashing (v002),
and narrowing counters from `u32` to `u8` (v003). The two efficiency-focused
sketch reshapings — cheaper hashing and narrower counters — won; the aging variant
lost.

## Patterns

- Winners both attacked sketch *cost* (per-op-time and metadata-size) while
  leaving residency logic and the admission test untouched, and both paid only a
  marginal hit-rate cost — confirming the sketch carries slack that can be traded
  for speed and memory cheaply.
- The loser touched *admission semantics* by decaying the frequency signal; the
  trace appears to have stable, low true frequencies, so aging erased useful mass
  and the byte-width counters never saturated. Signal-preserving changes beat
  signal-altering ones this cycle.
- Tail latency was the recurring soft spot: both an inline full-table sweep
  (v001) and the byte-narrow access pattern (v003) introduced p99 variance even
  when p50 improved.

## Open questions

- Is frequency aging genuinely a dead end here, or just mistuned? The decay
  window was pinned far too tight and the sweep was unamortized.
- How far can counter narrowing go — does 4-bit packing keep the memory win
  without the p99 regression seen at 8-bit?
- Does double-hashing's correlated-rows hit-rate dip compound with other
  sketch changes, or stay marginal?

## Continuation

Pursue the cheap sketch-efficiency levers further: combine the v002 hashing win
with v003 counter narrowing, and push toward 4-bit packing while watching p99.
Before abandoning aging, retry it with a much wider decay interval and an
amortized (non-inline) sweep. Keep favoring changes that preserve the frequency
signal and residency logic over those that alter admission.
