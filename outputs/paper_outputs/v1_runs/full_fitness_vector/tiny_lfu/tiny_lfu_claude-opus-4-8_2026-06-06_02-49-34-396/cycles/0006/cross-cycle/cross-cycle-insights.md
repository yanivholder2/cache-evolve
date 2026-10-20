# Cross-Cycle Insights

## Recurring wins

- Cost-cutting sketch reshaping pays repeatedly at little or no hit-rate
  cost: cheap multiply-xorshift hashing, narrowed (u8) counters, and
  compact integer/sentinel node links have all won. The sketch carries
  real slack that trades cleanly for speed and memory.
- Compact integer/sentinel links (`u32 != NIL`) strictly dominate
  `Option<usize>`: half the per-node footprint, identical hit-rate, and
  slightly better tail latency from cheaper branch tests.
- Composition of proven efficiency winners is additive — confirmed three
  times (cycles 3–5). Levers on disjoint structures (counter width,
  hasher, node layout, sketch width) stack their gains without stacking
  downsides.
- Sharpening the existing frequency signal beats adding admission
  structure: widening the sketch and conservative-update counting both
  lifted hit-rate, the former essentially for free.

## Recurring failures

- Changing admission *semantics/structure* keeps losing: frequency aging
  (mistuned, signal-erasing) and a small W-TinyLFU recency window (too
  small to buffer newcomers) both underperformed while costing metadata
  and eviction-path p99. Signal-preserving changes beat signal-altering
  ones every time it has been tested.
- Reshaping sketch allocation to recover the per-op tax does not work:
  flattening `Vec<Vec<u8>>` to one buffer left per-op time worse. The tax
  is footprint + within-row cache-line stride, a genuine floor, not a
  layout artifact.
- Buying back row independence (two real hashes) buys nothing here: the
  double-hashing "dip" is effectively zero on this trace; correlation was
  never the accuracy bottleneck, so it is pure speed cost.
- Tail latency (p99) is the persistent soft spot — most cost wins nudge it
  the wrong way (sweeps, sample walks, narrow-access patterns).

## Open questions

- Does conservative update still pay on the wider sketches, where
  collision over-count is already low, or does its hot-path min-scan tax
  then outweigh a smaller accuracy gain?
- Is there a combined sketch width/depth sweet spot, or do per-op and heap
  costs stack faster than accuracy gains beyond the current point?
- With layout and hash-independence ruled out as recovery levers, is the
  only remaining route to cheaper per-op a smaller/sparser sketch that
  trades back hit-rate?
- Is residency/admission logic now the only frontier with untapped slack,
  given cost/layout axes are tight?

## Per-policy notes

- CountMinSketch (TinyLFU-style) admission is the dominant policy and the
  source of most slack; signal-quality levers (width, depth, conservative
  update) are where hit-rate gains now come from, but each is now priced.
- Sampled-coldest victim eviction: gives only marginal lift and appears
  near-saturated — the binding constraint on eviction quality is the
  sketch *estimate*, not sample depth. Don't deepen sampling again without
  a sharper estimate.
- W-TinyLFU recency window: failed at ~1% window; only worth revisiting at
  a substantially larger fraction, otherwise pure overhead on this trace.
- Frequency aging: provisional dead end as tuned — only revisit with a far
  wider decay interval and an amortized (non-inline) sweep.
