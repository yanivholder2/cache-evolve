## Recurring wins

- Reallocating fixed budget toward the promotion-protected normal tier was the
  dominant hit-rate lever across cycles 0001–0002: the baseline over-provisioned
  the cold stash, and shifting capacity from stash to normal kept climbing from
  even split into normal-dominant territory before saturating.
- Protecting the warm working set from transient-key disturbance pays off
  regardless of mechanism (capacity, insertion position, promotion gate) — all
  point at one underlying baseline weakness rather than independent ones.
- Wins have been budget-conserving and invariant-preserving; gains came without
  any dimension regressing except where a policy was actively swapped.

## Recurring failures

- Stacking selective stash-admission gates (frequency gate, back-insertion) on
  top of the capacity reshape is redundant, not additive — once the protected
  tier is enlarged, most available benefit is already captured and gates yield
  steeply diminishing returns.
- Spending conserved budget on bucket depth/width instead of bucket count
  regresses: widening buckets while halving their count collides hot keys into
  fewer landing spots. Bucket addressability is a value to preserve.
- Pushing the normal/stash split past its peak no longer climbs and edges
  fractionally lower — the capacity axis is now confirmed saturated.

## Open questions

- With the split axis and protected-tier promotion mechanics both near
  exhaustion, where does the next real hit-rate source on this trace come from?
  Every geometry/promotion lever now returns slivers.
- Has the cohort over-fit to a single high-skew trace? Universal lever
  flattening may be a workload property, not a design property — validation on a
  less-skewed or churn-heavy trace is untested.
- Untouched protected-tier levers beyond capacity/eviction/promotion-position
  (e.g. segment count, admission/eviction dynamics) remain unexplored.

## Per-policy notes

- Recency / promote-on-hit on the protected tier suits the skewed trace well;
  inserting hot keys at MRU so they climb in one follow-up access helped, but
  only marginally because fast re-touch made the "earn your way up" delay cheap.
- LFU eviction on the protected tier is a dead lever — it regressed by retaining
  stale-but-frequent keys over the live working set.
