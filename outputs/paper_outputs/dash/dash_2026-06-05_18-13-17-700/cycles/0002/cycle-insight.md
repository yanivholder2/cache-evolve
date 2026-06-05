# Cycle 0002 Insight

## Cohort outcome

This cycle probed three follow-ups to the cycle-0001 stash-vs-normal theme:
pushing the bucket split further toward the protected tier, stacking a
selective stash gate on the even split, and swapping the protected tier's
eviction policy. The winner (v001) pushed the split past even toward
normal-dominant and kept gaining hit-rate; the gate-stacking and policy-swap
variants did not improve on it.

## Patterns

- The capacity-reallocation axis from cycle 0001 has not yet saturated:
  moving budget further into the promotion-protected normal tier (even →
  normal-dominant) still climbed hit-rate, so the optimum lies beyond the
  even point — the baseline stash was over-provisioned more than one step
  could correct.
- The two stash-protection levers proved redundant, not additive: once the
  protected tier was enlarged by capacity, layering a selective-admission
  frequency gate on top yielded steeply diminishing returns. Most of the
  available benefit is captured by *how much* room the protected tier gets.
- Changing *how* the protected tier ranks residents was a dead lever:
  forcing LFU eviction onto the normal tier regressed, since the trace's
  skew is already well-served by recency/promote-on-hit and LFU retained
  stale-but-frequent keys over the live working set.
- All movement stayed budget-conserving and invariant-preserving; the only
  regression came from the policy swap, not from any capacity reshape.

## Open questions

- Where exactly does the normal-dominant split stop paying off — is there a
  point where too small a stash starves admission and regresses? The curve
  is still rising but unbounded sampling hasn't found its peak.
- Are there protected-tier levers beyond capacity and eviction-policy that
  haven't been touched (e.g. promotion position, segment count)?

## Continuation

Keep sweeping the normal/stash split toward the normal tier to bracket the
hit-rate peak — v001 confirms the curve is still climbing, so push past 3:1
until it turns over, then back off. Drop two directions as settled: stacking
selective-admission gates on top of the capacity reshape (redundant), and
swapping the protected tier's eviction policy to LFU (regresses). If the
split curve flattens, shift to an untried protected-tier lever rather than
re-running admission filters.
