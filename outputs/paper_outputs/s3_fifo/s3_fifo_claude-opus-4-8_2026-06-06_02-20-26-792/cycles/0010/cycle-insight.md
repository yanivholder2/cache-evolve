# Cycle 0010 Insight

## Cohort outcome

Off the cycle-0009 momentum adaptive-split parent, this cycle probed two
fresh surfaces — the small admission queue's intra-ordering (eager
`MAX_FREQ` promotion in H01, move-to-front in H02) and a freshness-weighted
controller step keyed to ghost-match depth (H03). Only small-queue
move-to-front (v002) landed in the Winners, and only barely; eager promotion
was inert and freshness-weighting regressed.

## Patterns

- The proven recency-reordering discipline finally paid on the one queue it
  had never touched. Applying main's move-to-front to the small hit branch
  makes `evict_small` shed the least-recently-accessed candidate instead of
  the oldest-inserted one, converting a short tail of re-touched keys. But the
  gain is tiny and the lever is near-saturated: small is too small and churns
  too fast for most candidates to earn a second hit, so the strict `freq > 1`
  promotion gate, not insertion order, still sets the filter's ceiling. Note
  this softly revises the cycle-0008 read that small-queue move-to-front was
  inert — under the now-adaptive split it nudges positive, but only at the
  margin.
- Promotion *timing* is not a live lever. Advancing a key into main the moment
  it saturates `freq`, rather than at eviction, changed nothing: a `MAX_FREQ`
  key already clears the promotion test and lands in main at eviction anyway,
  so moving only the moment neither admits new keys nor alters residency. The
  binding constraint is which keys promote and how long they reside, not when.
- Ghost-match depth carries no steering signal beyond its `from_main`
  direction. Jumping straight to the step cap on a "fresh" front-of-ghost hit
  made the split move faster but not better, landing below the parent —
  overshoot, not sharpening. This reinforces cycle-0009: the momentum
  controller is already well-aimed and converges fast enough; modulating step
  magnitude by signal recency is a dead end on this trace.

## Open questions

- v002's small-queue ordering win is real but explicitly near-saturated and
  bounded by small's size and promotion gate. Is there any remaining headroom
  on the small filter, or is it now fully characterized?
- Three mechanism families (split adaptivity, ghost capacity, small-queue
  ordering/timing) have each now paid at most marginally or regressed. Is the
  configuration effectively converged for this workload?

## Continuation

Adopt v002's recency-aware (move-to-front) small admission queue as the new
parent — a small but real gain, the first time the ordering lever helped on
the small queue under the adaptive split. Drop eager `MAX_FREQ` promotion
permanently (pure timing, inert — promotion already happens at eviction) and
drop freshness-weighted controller step permanently (overshoots; ghost depth
carries no steering signal beyond `from_main`). Every probed axis is now at or
past saturation: split adaptivity, ghost depth/capacity/warm-start, weak-ghost
removal, small-queue ordering and promotion timing, eviction ordering, victim
sharpening, and insertion position are all exhausted or falsified. The next
cycle should seriously weigh declaring convergence; if it continues, it must
find a mechanism not yet touched rather than re-tune any of the above.
