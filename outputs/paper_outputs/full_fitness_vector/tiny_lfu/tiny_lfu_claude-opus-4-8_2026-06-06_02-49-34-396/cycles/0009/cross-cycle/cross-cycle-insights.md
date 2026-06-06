# Cross-Cycle Insights

## Recurring wins

- Cost/efficiency levers that preserve the frequency signal and admission
  logic pay reliably and nearly free: narrowed counters, a cheap
  multiply-xorshift hasher, and compact `u32`-sentinel node links each won
  on their target axis at zero-to-marginal hit-rate cost.
- Composition is additive. Stacking proven winners (fast hasher + narrow
  counters, compact layout + wider sketch) repeatedly delivered the sum of
  the gains because the changes touch disjoint structures — confirmed
  across several cycles.
- Sharpening the *existing* signal beat adding new structure. Widening the
  sketch (more columns → fewer collisions) was the strongest hit-rate
  mover in the early-to-mid cycles, and came free until cache-locality
  cost set in.
- Compact integer/index encodings strictly dominate `Option`-based links:
  smaller footprint, equal hit-rate, and slightly better tail (`!= NIL`
  cheaper than `match`).

## Recurring failures

- Counter aging/decay is a settled net loss on this trace. Tried tight,
  tried wide+amortized — it erases earned mass on a stable hot set faster
  than it adds contrast, dropping hit-rate while taxing every increment.
- Added admission/eviction *structure* keeps being inert: a small
  recency window (W-TinyLFU), SLRU probation/protected split, and a
  one-count admission margin all failed because the existing
  `incoming_freq > victim_freq` gate already does that filtering. The
  hit-rate ceiling is set by admission quality, not list shape.
- Edits to the per-op hot path buy nothing once the sketch is wide:
  conservative update (gain at factor-4 vanishes at factor-16), branchless
  saturating increment, and removing a row all sat below the noise floor
  or regressed per-op time.
- Row-hash independence (real second hash vs. double-hashing) and flattening
  the sketch allocation both resolved negatively: the correlation was never
  the accuracy bottleneck, and the wide-sketch per-op tax is a genuine
  footprint/cache floor, not a layout artifact.

## Open questions

- Is hit-rate effectively saturated at the incumbent base? Every named
  mover — estimate rule, geometry, admission margin, eviction structure —
  has now been ruled flat on the wide base.
- The eviction *walk* / victim sampling is the one surface not yet mined
  for tail/per-op savings — does it hold cost wins without touching
  hit-rate?
- Is there any admission-side edit that is neither a constant/margin tweak
  nor a structural overlay, given admission is repeatedly the binding
  constraint?

## Per-policy notes

- **CountMinSketch (frequency signal):** carries real slack for
  cost-cutting (narrow counters, cheap hash) but its accuracy knobs have
  hard limits — width saturates around factor-16, depth has a floor at 4
  (trimming below leaks aliased estimates), and conservative update only
  helps while collisions are common.
- **TinyLFU admission gate:** the dominant determinant of hit-rate and
  repeatedly the binding constraint; it already absorbs the work that
  windowing/segmentation/margin schemes try to add, leaving them inert.
- **Sampled-coldest eviction:** victim-sample depth is accuracy-saturated
  (the sketch estimate, not walk length, bounds eviction quality); useful
  only as a p99-tail/cost knob, and cheapest when it shares the single
  cached hash fold.
