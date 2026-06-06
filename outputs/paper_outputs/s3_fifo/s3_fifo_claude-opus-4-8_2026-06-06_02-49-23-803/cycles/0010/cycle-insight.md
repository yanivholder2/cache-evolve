# Cycle 0010 Insight

## Cohort outcome

All three variants branched off cycle 0009's v003 and fired one mechanism
each at a different scored axis: relocating the last owned field to make
`Entry` zero-sized (metadata), eliding the saturated freq write (per-op
latency), and lifting the second-chance freq ceiling (hit-rate). Two
produced genuine movement — metadata driven to 0 and the long-frozen
hit-rate plateau finally cleared — while the latency lever backfired. All
three were retained as Winners.

## Patterns

- The metadata axis is now self-evidently exhausted *and* exposed as a
  measurement artifact. Carrying the field-relocation technique to its
  limit relocated key, freq, and finally value out of `Entry` until the
  type is a legal ZST reporting size 0, with hit-rate bit-identical and
  real per-slot memory (value + freq still in the map tuple) untouched.
  The dimension can no longer distinguish saved memory from relocation
  bookkeeping.
- The hit-rate plateau was a retention ceiling, not a structural limit.
  Loosening only the *retention* cap (raising MAX_FREQ) while leaving the
  strict admission gate intact converted already-proven reuse into more
  eviction-pass survival, edging hit-rate past its many-cycle floor for
  free on memory and latency. This confirms the recurring cross-cycle
  signal that hot keys had unexploited headroom under a too-low ceiling.
- The remaining per-op-time lever is a dead end. The median get was
  already at the measurement floor, so eliding the freq store bought no
  p50 win; worse, trading a branchless clamped store for a value-dependent
  branch introduced misprediction jitter that widened the tail. Latency is
  confirmed floored, not merely unaddressed.

## Open questions

- With metadata reporting 0 from a ZST, should the scoring be changed to
  measure real per-slot footprint (the map tuple) rather than a
  relocatable type size that no longer tracks anything?
- How far can the retention ceiling rise before it retains gone-cold keys
  and regresses — is there an optimum between the old cap and the value
  tried, or further headroom above it?
- The unscored container-memory frontier (separate ghost set vs.
  folded-tombstone) has now gone four cycles flagged but never directly
  contested — is it the only axis left with real cost to differentiate?

## Continuation

Treat metadata-size as closed and discredited: it now reports 0 from a
zero-sized type, proving the axis measures field location, not memory — do
not mine it further, and consider whether the metric itself should be
redefined to capture real per-slot footprint. Per-op latency is also
closed: the median is at the measurement floor and the one remaining store
is the wrong lever (it costs tail jitter for no median gain). The one live
scored thread is hit-rate via *retention strength* — the freq-ceiling
lift just broke a long plateau, so probe for its optimum without touching
the load-bearing admission gate. If a fresh axis is needed, finally
contest the unscored container-memory tradeoff that prior cycles have
named but never tested.
