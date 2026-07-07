# Cross-Cycle Insights

## Recurring wins

- **Widening bucket geometry improves hit-rate.** Repartitioning total capacity into
  fewer, larger sets gives each per-set LRU a bigger eviction pool, reducing premature
  eviction of still-live items. This held across cycles 0001 and 0002, though the
  marginal gain diminishes as buckets grow; the first widening step was the dominant
  payoff.
- **Targeting scan pollution pays off.** Both cold-first eviction (selecting
  counter==0 items as eviction candidates) and mid-point insertion attacked the same
  root cause — scan traffic displacing hot items — and both produced positive
  hit-rate outcomes. The eviction-side intervention was larger in magnitude.
- **Geometry widening enables pollution-filtering strategies.** Cold-first eviction
  depends on a resident pool of cold items large enough to absorb scan traffic; the
  wider buckets from the geometry win are a prerequisite for that pool to exist
  reliably.

## Recurring failures

- **O(1) LRU promotion via swap-to-back hurts hit-rate.** Swapping the accessed item
  to the MRU position rather than performing a full O(n) relink corrupts the recency
  gradient across intermediate slots; the resulting eviction quality regression was
  large and repeatable. Strict ordering is load-bearing for ClassicLRU.
- **Pushing geometry beyond the initial widening yields diminishing returns.** Moving
  from a 4× to a 16× bucket factor produced only a fraction of the first step's gain.
  The workload's hot set fits comfortably in moderately wide buckets; further geometric
  expansion does not pay for itself.

## Open questions

- Do cold-first eviction and mid-point insertion stack additively, or do they address
  overlapping portions of the same scan-pollution budget?
- Does the linear scan across bucket slots to find a cold candidate add measurable
  per-op latency at scale, especially in wide buckets under high load?
- As the cache warms over a long trace, does the cold pool thin out enough to trigger
  the fallback to plain LRU frequently, and if so, does the hit-rate gain degrade?
- Does a frequency threshold higher than 1 (e.g., protect items with counter ≥ 2)
  improve hot/cold separation or just reduce the available eviction pool?
- Do the geometry and scan-pollution gains generalise to traces beyond LIRS workload-a,
  or are they specific to its locality pattern?
- Are the per-op-time and metadata-size evaluators reachable? Structural changes such
  as timestamp removal cannot receive a real verdict until those dimensions are
  measured.

## Per-policy notes

- **ClassicLRU:** The dominant bottleneck on LIRS workload-a was narrow bucket
  geometry combined with scan pollution. Both have been partially addressed: geometry
  widening (cycle 0001 winner) and cold-first eviction (cycle 0002 winner) are the
  two confirmed improvements. Mid-point insertion is a promising secondary lever,
  unconfirmed in combination. O(1) swap-to-back promotion is a confirmed anti-pattern
  and should not be revisited unless a recency-preserving O(1) variant emerges (e.g.,
  clock approximation, segmented LRU).
