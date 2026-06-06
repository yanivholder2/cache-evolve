## Recurring wins

- Cost-cutting levers that leave admission semantics and residency logic
  untouched keep paying off at marginal or zero hit-rate cost — fast
  multiply/xorshift hashing, single-hash double-hashing, and counter
  narrowing all confirm the sketch carries exploitable slack.
- Proven efficiency winners compose additively rather than competing:
  stacking the cheap hasher with narrowed counters cost no extra per-op
  time and no hit-rate.
- The biggest clean wins came from attacking axes nobody had touched —
  re-encoding node links to shrink per-entry metadata was regression-free,
  because every prior variant had crowded the sketch heap and ignored node
  layout.

## Recurring failures

- Signal-altering admission changes lose. Frequency aging is the standing
  example: the trace appears to have stable, low true frequencies, so decay
  erases useful mass and the counters never saturate. Signal-preserving
  changes consistently beat signal-altering ones.
- Single-lever probes off the shared baseline have thinning returns — once
  efficiency/layout slack is harvested, isolated tweaks move one axis only
  by nudging another the wrong way.

## Open questions

- With hasher, counter width, and node layout all tight, where is the next
  pool of slack? Residency/admission logic looks like the only remaining
  frontier.
- Is frequency aging a genuine dead end or just mistuned? It has never been
  retried with a wide decay window and an amortized (non-inline) sweep.
- Is the correlated-rows hit-rate dip from double-hashing worth buying back
  row independence, or is it permanently subsumed by the speedup it enables?
- Can recurring p99 costs be amortized away (e.g. precomputed eviction-path
  estimates) rather than accepted as the price of each win?

## Per-policy notes

- Strict-LRU-head eviction is the cheap, tail-stable baseline.
- Sampled-coldest eviction lifts hit-rate but only marginally, and at the
  heaviest tail cost of its cohort when each candidate pays a full hash.
  Migrating the sample walk onto a single cheap fold tamed the eviction-path
  p99, turning the tail cost into a net positive — its viability hinges on
  sharing one hash and tuning sample width.
