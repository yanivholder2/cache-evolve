## Recurring wins

- Explicit segmentation with a promote-on-first/second-hit rule is the
  durable breakthrough: demanding proof of reuse before granting protection
  beat both pure-LFU victim selection and single-position LRU admission, and
  every later baseline has been built on it.
- Tuning *which* probationary entry dies is high-leverage at zero metadata
  cost: reversing probation to oldest-first (FIFO) gives a fresh key a full
  lap to earn promotion and lifted hit-rate on its own.
- The earliest, cleanest gains came from intra-bucket victim/admission logic
  (frequency-first eviction over pure-LRU), leaving routing, capacity
  invariants, and the policy enum untouched.

## Recurring failures

- Anything that *erodes durable, earned protection* regresses on this
  collision-bound trace: counter aging, front-admission composition,
  second-hit promotion thresholds, capped/demoting protected segments. Once a
  key proves hot it wants to stay protected ("once-hot immortality").
- "Recently evicted" is not a reliable proxy for "hot": ghost-ring fast
  re-entry into the protected segment let weakly-warranted keys seize space
  they did not earn and regressed.
- Approximating recency away (CLOCK one-bit reference) underperforms full
  positional ranking in small buckets.
- Orthogonal levers that each helped alone do not necessarily compose —
  frequency-first eviction plus probationary front admission actively
  conflict, making fresh keys the immediate victim.
- Geometry coarsening (wider buckets) gave the smallest gain and hit
  diminishing returns early.

## Open questions

- Is hit-rate now at its ceiling on this trace? Admission, promotion,
  protected-eviction signal, and re-entry have all been ruled out.
- Does the one genuinely untested hit-rate axis — looser/larger protected
  sizing and bucket-geometry interaction — move the needle, since *enlarging*
  protection (rather than tightening or re-routing it) has never been tried?
- Cost-side wins (reorder-free tiebreak, no-op reorder elision) are real but
  invisible to a hit-rate-only fitness and always tie. Is a per-op-time
  fitness axis warranted before spending more cohort slots on them?

## Per-policy notes

- Pure-LRU victim selection: superseded early by frequency-first eviction.
- Frequency/LFU signals: useful as a promotion flag, but as a victim or
  re-entry signal they let `==1` newcomers dominate and kill rising keys;
  banked counts do not separate hot from rising better than protected-LRU
  recency.
- Segmented-LRU (FIFO probation + promote-on-first-hit): the strongest scheme
  found and the standing baseline; behavioral logic appears saturated for
  hit-rate. The protected segment is load-bearing and never saturates, so its
  cap-and-demote branch is effectively dead code on this trace.
- CLOCK / one-bit recency: too coarse for small buckets; loses warm keys.
