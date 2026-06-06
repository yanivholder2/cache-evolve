# Cross-Cycle Insights

## Recurring wins

- Cost-cutting levers that leave admission semantics and residency logic
  untouched keep paying off: narrowing counters, cheaper hashing, and
  compact node layout all win at zero-to-marginal hit-rate cost. The
  count-min sketch carries persistent slack that trades cleanly for
  per-op time and metadata size.
- Compact integer node layout (u32 sentinels replacing `Option<usize>`)
  has won every time it was tried — it halves per-entry footprint with
  bit-identical hit-rate and even slightly better tail latency, since
  `!= NIL` tests beat `match`-on-`Option`.
- Composition is additive, not competing: stacking proven efficiency
  winners (fast hasher + narrowed counters, eviction onto the single
  cheap hash) has reliably preserved each lever's gain without stacking
  downsides.
- When the goal is hit-rate, sharpening the *existing* signal (e.g.
  widening the sketch to cut collisions) beats adding new structure, and
  it tends to come nearly free because the hot path already strides the
  sketch heap with one cached hash.

## Recurring failures

- Changes that alter admission *semantics or structure* keep losing:
  frequency aging erased useful low-frequency mass, and a small
  W-TinyLFU recency window never buffered enough newcomers to help while
  costing metadata and tail latency. Signal-preserving changes have
  consistently beaten signal-altering ones.
- Tail latency (p99) is a chronic soft spot: nearly every primary win has
  nudged p99 the wrong way (full-table sweeps, narrow-counter access,
  per-eviction sample walks) until explicitly amortized.

## Open questions

- Does sketch widening keep buying hit-rate past the tested factor, or
  does collision recovery plateau quickly? Same for counter precision.
- Is residency/admission logic the only remaining frontier now that
  layout, counter width, and hasher are all tightened together?
- Can the recurring p99 cost be amortized away as a rule (precomputed
  estimates, shared single-hash folds) rather than re-paid per lever?
- Is the admission-window idea simply mismatched to this trace's
  cold-start profile, or does it have merit only at a much larger window
  fraction?
- Is the correlated-rows hit-rate dip from double-hashing worth buying
  back, or is it permanently subsumed by the speedup it enables?

## Per-policy notes

- LRU-head eviction: the baseline; cheap and tail-friendly.
- Sampled-coldest victim walk: lifts hit-rate only marginally and at the
  cohort's heaviest eviction-path tail cost when run over the expensive
  hash; moving it onto a single cheap double-hash fold tamed the tail and
  turned it net positive. Treat its lift as provisional and width-tunable.
- W-TinyLFU admission window: failed at ~1% window — too small to
  accumulate frequency before the gate, pure overhead. Only revisit at a
  substantially larger window fraction.
- Frequency aging: lost outright on this trace's stable, low true
  frequencies; decay erased signal the byte counters never needed.
