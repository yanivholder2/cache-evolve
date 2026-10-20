# Cycle 0004 Insight

## Cohort outcome

The cohort tried to push past the established hit-rate ceiling at the
16-byte floor from three angles: repairing the stateful reuse-bit path's
quadratic tax (v001), compounding explicit protection onto stateless
front-admission (v002), and widening the probation window via midpoint
insertion (v003). The two Winners each held the line rather than advancing
it — v001 recovered clean tails without moving hit-rate, v002 held the
metadata floor but gained nothing — while the midpoint geometry probe
regressed and lost.

## Patterns

- The index-preserving swap the prior cycle called for works exactly as
  predicted: mirroring each policy's single-element relocation onto the
  parallel reuse vector collapses the p99 tails and put p50 that the
  quadratic key-matched rebuild had inflated, with hit-rate held bit-for-bit
  identical. The parent's only real tax was the rebuild algorithm, not the
  scan-resistance mechanism.
- Explicit reuse protection and stateless front-admission are the same lever
  expressed twice, not compounding ones. Under move-to-back-on-hit, the
  frontmost slot is *always* the probationary one, so an added reuse bit
  resolves to the eviction index the position ordering already selects —
  v002 came out behaviorally indistinguishable from its parent. Protection
  adds value only over a policy that does not positionally segregate reused
  items.
- The admission-geometry axis has a sharp optimum at the front slot.
  Midpoint admission is strictly worse: the longer proving window it grants
  hot-but-unrehit items is dwarfed by the cost of letting never-reused scan
  items linger above the eviction slot, dropping hit-rate and nudging tails
  up. Front's value is precisely that a fresh unproven miss is the next
  thing evicted.
- The hit-rate ceiling held against every angle this cycle. Two independent
  attempts to exceed it at the floor produced no gain — evidence the current
  scan-resistance mechanisms are saturated, not that a better encoding of
  the same signal remains to be found.

## Open questions

- v001 flagged a metadata regression off the 16-byte floor that its own diff
  cannot explain (`Item` is still key+value; the reuse vector lives on the
  unmeasured bucket). Is this an overlay/inheritance artifact rather than a
  real layout change?
- If explicit protection and front-admission are redundant under the current
  policy, is there hit-rate headroom only reachable by changing the base
  policy's ordering itself — or is this trace's ceiling genuinely structural?
- Every stateless and stateful path now converges on the same hit-rate.
  Where does the remaining headroom, if any, actually live?

## Continuation

Adopt v001's index-preserving swap as the clean stateful incumbent — it
delivers the floor-level hit-rate with tails matching the stateless path.
Stop stacking explicit reuse protection onto front-admission: the two levers
are redundant under move-to-back-on-hit, so future protection ideas must
override a policy that does *not* positionally segregate reused items to
have any effect. Treat the front slot as the settled optimum on the
admission-geometry axis and drop midpoint/wider-probation variants. Next
cycle should first resolve v001's unexplained metadata regression (likely an
overlay artifact), then, since same-signal re-encodings have plateaued,
probe whether any hit-rate headroom requires changing the base policy
ordering rather than layering state on top of it.
