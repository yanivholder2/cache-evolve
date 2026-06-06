## Recurring wins

- Cost/layout efficiency levers that preserve admission semantics paid off
  repeatedly and cleanly: narrowing counters, cheaper multiply-xorshift
  hashing, and re-encoding intrusive node links as integer sentinels instead
  of `Option`. These touch disjoint structures from the residency logic and
  banked footprint, tail, or per-op savings at zero or marginal hit-rate cost.
- Composition is additive — confirmed across multiple cycles. Stacking proven
  efficiency winners (fast hasher + narrow counters, compact layout + wide
  sketch) delivered each lever's gain at the other's budget, because the
  changes touch non-overlapping structures.
- Sharpening the existing frequency signal beat adding new admission/eviction
  structure. Widening the count-min sketch to cut collisions was the strongest
  hit-rate mover found, and it came essentially free on the hot path.

## Recurring failures

- Counter aging / frequency decay is a settled net loss on this trace, retried
  several ways (tight and wide intervals, amortized sweep). The hot set has
  stable, well-separated true frequencies, so decay erases earned mass faster
  than it refreshes contrast while taxing every increment.
- Structural admission/eviction machinery underperforms: a small W-TinyLFU
  recency window and an SLRU probation/protected split each moved hit-rate by a
  sliver while charging metadata and per-op cost — the existing frequency
  admission gate already does their job.
- Constant/margin tweaks to the increment loop and admission threshold are
  inert: branchless saturating add, single-count admission margins, and
  conservative update (on the wide sketch) all landed flat. Edits that touch
  the hot path pay unconditional per-op time for no hit-rate.
- Per-op speedups from trimming the eviction walk never materialized above
  benchmark noise — depth cuts, sample trims, floor early-exit, and random
  direct-index sampling. The walk is not the dominant put-path cost.

## Open questions

- Is node *layout* — physically clustering victim candidates so they are
  contiguous in `nodes` — a real per-op lever, or is the order-uncorrelated
  cache-miss pattern intrinsic to any recency-ordered structure?
- Has hit-rate effectively saturated at the incumbent base? Every named mover
  (estimate rule, geometry, admission margin, eviction structure) has now been
  ruled flat on the wide-sketch base.
- If hit-rate is saturated, does the frontier shift entirely to banking
  cost-axis wins (footprint, p99 tail) at fixed hit-rate?

## Per-policy notes

- CountMinSketch admission is the dominant policy lever and carries most slack:
  signal quality (width, conservative update) moves hit-rate; geometry knobs
  (depth, width) and sample depth saturate for accuracy and become pure cost
  levers. Sketch depth has a hard floor at 4; below it aliasing leaks into
  admission. The wide sketch's per-op tax is a genuine floor (footprint + hash
  stride), not a layout artifact, and row-hash independence buys no accuracy.
- LRU recency is load-bearing, not redundant: the per-get move-to-tail keeps
  re-accessed hot keys out of the oldest-inserted sample window, and the
  frequency gate alone does not protect them — removing it costs hit-rate
  sharply.
- Sampled/segmented eviction (sampled-coldest walk, SLRU) gives at most
  marginal hit-rate lift because victim quality is sketch-estimate-bound, not
  sample-source- or list-shape-bound; its main historical value was taming the
  eviction-path p99 once moved onto a single cheap hash.
