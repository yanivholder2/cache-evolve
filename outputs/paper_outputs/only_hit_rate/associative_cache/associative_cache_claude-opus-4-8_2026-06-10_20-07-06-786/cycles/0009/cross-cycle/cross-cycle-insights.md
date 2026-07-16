# Cross-Cycle Insights

## Recurring wins

- **Segmentation with proof-of-reuse beats single-scalar tuning.** The
  durable line of progress was: frequency-first victim selection →
  explicit segmented-LRU (promote-on-second-hit) → FIFO probation order.
  Demanding evidence of reuse before granting protection, then giving
  rising keys a full probation lap to earn it, repeatedly lifted hit-rate.
- **Promote straight to protected-MRU, single hit, then keep forever.**
  One probationary hit is sufficient proof on this trace; promoted keys
  belong at the MRU end and benefit from "once-hot immortality." Every
  scheme respecting this held its gains.
- **Geometry is a real, independent lever.** After behavioural saturation,
  a single capacity-conserving bucket-count halving (wider associativity)
  produced the only fresh hit-rate gain in many cycles.
- **Behaviour-preserving cost rewrites are always safe to bank.** O(1)
  eviction scans, reorder-free tiebreaks, rotate-based get/put, and smaller
  metadata are provably byte-identical and never regress.

## Recurring failures

- **Protection erosion always loses.** Counter aging, second-hit promotion
  thresholds, protected-segment caps with demotion, CLOCK recency
  approximation, LFU-min fallback, ghost-ring fast re-entry, and elite
  sub-tiers all regressed — each made earned protection harder to get or
  easier to lose on a collision-bound trace.
- **The two original levers do not compose.** Front/probationary admission
  stacked onto frequency-first eviction makes a fresh key both oldest and
  min-frequency, so it is evicted immediately.
- **Geometry overshoots past one halving.** A second halving (too-wide
  buckets) and absolute associativity targets (degenerate fixed width) both
  regressed; the sweet spot is bracketed at exactly one halving.
- **Looser/larger or re-tiered protected sizing is net-negative** in every
  form tested, including non-eroding reserved-slot variants.
- **Routing quality offers no headroom** — SipHash is already uniform on
  this trace, so stronger finalizers only reshuffle collisions.

## Open questions

- Is hit-rate now fully at its ceiling? Admission, promotion, eviction
  signal, re-entry, protected sizing/tiering, routing, and bucket width
  have all been exhausted.
- Should a per-op-time / metadata-size fitness dimension be added? Cost-side
  wins keep recurring and keep landing as undifferentiated ties because the
  hit-rate-only vector cannot see or reward them — deferred for four+ cycles.
- Do the independent, stackable cost rewrites change anything observable
  once all adopted, or do they remain collectively invisible to fitness?
- Is there any untested trace-structure axis left, or should the loop be
  declared solved and stopped?

## Per-policy notes

- **Segmented-LRU (the established baseline):** the strongest scheme found.
  FIFO (oldest-first) probation + single-hit promote-to-MRU + immortal
  protected segment is settled and load-bearing. Do not tighten, cap,
  re-tier, or re-seat below MRU.
- **Pure LRU:** the starting point; superseded once frequency/segmentation
  shielded the working set from one-hit-wonders and scan traffic.
- **LFU / frequency scalars:** useful only as the bridge to segmentation;
  as a standalone victim signal or fallback it lets `==1` newcomers dominate
  and kills freshly-promoted rising keys.
- **CLOCK / one-bit recency:** coarse reprieve underperforms full positional
  ranking in small buckets — recency cannot be safely approximated away.
- **Geometry (orthogonal to policy):** one halving optimal; bracketed above
  and below. Wider buckets raise per-eviction scan cost, invisible to
  current fitness.
