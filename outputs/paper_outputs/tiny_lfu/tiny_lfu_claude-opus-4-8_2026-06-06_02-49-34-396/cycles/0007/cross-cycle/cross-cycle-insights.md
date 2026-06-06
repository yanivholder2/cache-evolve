# Cross-Cycle Insights

## Recurring wins

- Composition is additive: stacking independently-proven efficiency/layout
  winners (narrow counters + fast hasher + compact links + wider sketch)
  delivered each lever's gain with no extra cost, confirmed across multiple
  cycles — because the changes touch disjoint structures (sketch heap vs node
  layout).
- Cheap sketch-efficiency levers that preserve admission semantics — narrowing
  counters, swapping in a multiply/xorshift hasher — repeatedly won at zero or
  marginal hit-rate cost, confirming the sketch carries real slack.
- Compact integer-sentinel node links (u32 NIL instead of `Option<usize>`)
  strictly dominated: halved per-node footprint, bit-identical hit-rate, even
  slightly better tail latency. The metadata-size axis was the cheapest because
  nobody had touched node layout earlier.
- Sharpening the *existing* frequency signal beats adding new policy structure:
  widening the sketch (fewer collisions) and conservative update (tighter
  estimate) are the only changes shown to actually move hit-rate.

## Recurring failures

- Counter aging / frequency decay loses on this trace (cycles 0001 and 0007,
  even tuned wide with an amortized sweep): the hot set is stable and low-true-
  frequency, so decay erases earned mass faster than it refreshes contrast while
  taxing every increment. Settled as a dead end — do not revisit.
- Structural admission machinery underperforms relative to signal-sharpening: a
  small (~1%) W-TinyLFU recency window could not buffer enough newcomers to gate
  on, and cost metadata bytes plus eviction-path p99.
- Recovering the wide-sketch per-op tax by re-shaping memory failed — flattening
  the allocation left per-op time unchanged. The cost is footprint and within-row
  hash stride spilling cache lines, a genuine floor price, not a layout artifact.
- Buying back row independence with two real hashes bought no accuracy on this
  trace (row correlation was never the bottleneck) — pure added speed cost.

## Open questions

- Is there any remaining lever that sharpens the wide sketch's estimate itself —
  the one thing demonstrated to move hit-rate — that hasn't been ruled flat?
- Does conservative update still pay on the wider (factor-16) sketch where
  collision over-count is already low, or does its broad hot-path scan tax
  outweigh a smaller accuracy gain?
- Is there a combined sketch width/depth sweet spot, or do per-op and heap costs
  stack faster than accuracy gains past current width?
- Is the larger sketch's cache-locality per-op tax the permanent floor price of
  sharper admission, or recoverable by some untried means?

## Per-policy notes

- LRU-head eviction (baseline residency): cheap and stable; left untouched in the
  early winners, which only attacked sketch cost.
- Sampled-coldest victim walk: lifts hit-rate only marginally and adds put p99
  tail; the tail was tamed by sharing one cheap fold across candidates. Sample
  depth is now accuracy-saturated — deepening the walk barely helps and adds tail,
  so the sketch estimate, not sample width, binds eviction quality.
- CountMin sketch (admission signal): the dominant policy surface. Width is the
  strongest hit-rate lever (collision recovery); depth is accuracy-saturated on
  the wide sketch and tunable only for footprint; conservative update is the live
  estimate-sharpening candidate. Efficiency knobs (u8 counters, fast/single
  hasher) trade speed/memory for negligible hit-rate.
- W-TinyLFU recency window: provisional/negative at small window fraction; only
  worth revisiting at a substantially larger window.
