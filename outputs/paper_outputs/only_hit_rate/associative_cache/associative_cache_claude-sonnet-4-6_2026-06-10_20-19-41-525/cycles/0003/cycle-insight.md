# Cycle 0003 Insight

## Cohort outcome

The cohort tested three refinements of the cold-first eviction strategy: stacking
mid-point insertion on top of the cycle 0002 winner, tightening the cold threshold
from counter==0 to counter≤1, and applying cold-first eviction on narrow-bucket
geometry via mid-point insertion. The stacked combination (v001) won on a marginal
hit-rate gain; threshold tightening (v002) regressed; and the narrow-geometry
cold-first variant (v003) improved over its parent without being selected, because the
wide-bucket baseline it competed against started from a higher absolute floor.

## Patterns

- **The `counter == 0` eviction boundary is already well-calibrated.** Raising the
  warmth threshold to require two accesses before shielding an item caused genuine
  hot-set members to be evicted prematurely. On workload-a a single access is a
  reliable signal of future reuse, not scan noise; the threshold tuned in cycle 0002
  should be treated as settled for this trace.
- **Cold-first eviction and mid-point insertion are weakly additive when the eviction
  mechanism already dominates.** Stacking both together on wide-bucket geometry
  produced only a marginal gain over cold-first eviction alone, because cold-first
  selection already targets counter==0 items regardless of their vector position —
  insertion site becomes a tiebreaker rather than a primary lever.
- **Mid-point insertion can substitute for bucket widening as a cold-item concentrator.**
  Cold-first eviction delivered a hit-rate improvement even on narrow 16-slot buckets
  when mid-point insertion was present. This challenges the cycle 0002 conclusion that
  geometry widening is a necessary prerequisite; the two mechanisms appear to solve the
  same underlying problem — ensuring enough cold candidates exist per bucket — via
  different means.

## Open questions

- Does the mid-point-insertion-as-geometry-substitute trade-off hold under workloads
  with different locality profiles, or is it specific to workload-a's scan pattern?
- What is the per-op latency cost of mid-point insertion's O(n) `vec.insert` relative
  to the cold-first scan; is the marginal hit-rate gain worth the combined cost?
- At what bucket size does mid-point insertion's substitution effect break down — i.e.,
  is there a minimum slot count below which neither trick maintains enough cold
  candidates?

## Continuation

The cold-first + wide-bucket baseline remains the strongest configuration found so far,
with mid-point insertion adding a small incremental win. Before stacking further
mechanisms, next cycle should instrument per-op latency for both v001 and v003 to
determine whether the narrow-geometry + mid-point path offers a better hit-rate/latency
trade-off than the wide-bucket path. The discovery that geometry widening and mid-point
insertion are substitutable opens a new comparison axis worth probing directly. Drop
threshold-tightening hypotheses — the `counter == 0` boundary is confirmed for this
workload.
