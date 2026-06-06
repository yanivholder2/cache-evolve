# Cross-Cycle Insights

## Recurring wins

- Cost levers that preserve admission semantics and residency logic keep
  paying: cheap multiply/xorshift hashing, narrowed counters (u32→u8), and
  compact u32-sentinel node links all won at zero-to-marginal hit-rate cost,
  confirming the sketch carries exploitable slack.
- Composition of proven winners is additive when the changes touch disjoint
  structures (confirmed across cycles 3–5): stacked layout + hasher + wider
  sketch delivered each lever's gain without stacking downsides.
- Sharpening the *existing* signal beats adding new admission machinery:
  widening the count-min sketch quartered collisions and sharpened both the
  admission test and victim choice, essentially free on the hot path.
- The cheapest wins came from axes nobody had touched yet (node/metadata
  layout) — re-encoding intrusive links as integer sentinels strictly
  dominated `Option<usize>` in size and tail latency.

## Recurring failures

- Changes that alter admission *semantics or structure* keep losing:
  frequency aging (c1) erased useful low-frequency mass on a stable trace,
  and a ~1% W-TinyLFU recency window (c4) was too small to buffer newcomers
  before the gate — pure overhead.
- Eviction-side sampling shows diminishing returns: sampled-coldest victim
  walks and deeper victim samples give only marginal hit-rate lift while
  adding eviction-path p99; depth appears near-saturated.

## Open questions

- Does the W-TinyLFU / recency-window idea have merit at a substantially
  larger window fraction, or is it fundamentally mismatched to this trace's
  cold-start profile?
- Is there a sketch width/depth sweet spot, or do per-op and heap costs
  start outrunning the collision-recovery accuracy gains?
- Now that signal gains have crossed from free to priced, is the sketch's
  cache-locality / per-op tax recoverable via layout work, or is it the new
  floor price of sharper admission?
- Is the sketch estimate (not sample depth) the binding constraint on
  eviction quality — and is buying back row independence from double-hashing
  worth the speed it costs?

## Per-policy notes

- CountMinSketch admission: carries persistent slack — counter width and
  hasher cost can be cut cheaply; width/depth sharpen the signal but now
  each costs per-op time or heap. The correlated-rows dip from
  double-hashing is the one standing, tolerated cost.
- LRU / sampled-coldest eviction: strict-LRU-head replacement is the stable
  base; sampled and deeper-sampled victim selection lift hit-rate only
  marginally and at a tail cost, and appear saturated for this trace.
- W-TinyLFU recency window: failed at small (~1%) fraction; only revisit
  with a much larger window, otherwise treat as overhead.
- Node/metadata layout: integer-sentinel links dominate Option-based links
  on size and tail; this axis is now largely harvested.
