# Cycle 0006 Insight

## Cohort outcome

After several cycles of behavioral saturation, this cohort finally found a
hit-rate lever by leaving the bucket logic alone and changing *geometry*: v001
halved the bucket count and doubled bucket size (capacity conserved), winning
on associativity alone. The two behavioral variants both regressed — seating
promotions at the protected-LRU boundary (v002) and reviving two-hit promotion
inside wider buckets (v003) — so the segmented logic again stands unimproved
while the routing geometry moved the needle.

## Patterns

- The breakthrough came from outside the admission/promotion/eviction logic
  entirely. The trace was mildly associativity-starved: one bucket-count
  halving let each collision set hold more of the hot working set without
  eroding any earned protection, confirming geometry is the live axis that
  behavioral tuning is not.
- The two regressions reaffirm the multi-cycle protection-erosion boundary
  from a fresh angle. Seating promotions at the protected-LRU end (v002) made
  rising keys re-earn MRU standing and they got evicted sooner; the parent's
  promote-straight-to-MRU placement is better here.
- Wider buckets did not rehabilitate the known-bad two-hit threshold (v003):
  a longer probation lap still does not buy back keys that a stricter promotion
  bar strands. The two-hit regression is intrinsic to demanding a second proof
  of reuse on this trace, not an artifact of narrow baseline geometry.
- The win is described as small, suggesting the regime is near its
  associativity sweet spot — a second halving would likely trade routing away
  for diminishing returns.

## Open questions

- Is one halving the optimum, or is there a slightly different split (or a
  geometry that interacts with segment sizing) that extends the gain before
  routing loss dominates?
- Does the wider-bucket win change the cost picture — the larger per-bucket
  scan is invisible to hit-rate-only fitness but is a real per-op cost; is a
  time axis now worth adding?
- With geometry now confirmed live but behavioral logic saturated, are the two
  axes independent, or does the best segment policy shift once buckets widen?

## Continuation

Adopt v001's single-halving geometry on top of the FIFO-probation segmented-LRU
as the new baseline — this is the first hit-rate gain in several cycles and the
geometry axis is now the most promising direction. Probe geometry further but
cautiously: test whether the sweet spot is exactly one halving or admits finer
tuning, and whether segment sizing interacts with the wider buckets. Stop
reviving the two-hit threshold (now failed at both geometries) and stop
re-seating promotions below MRU — both join the long protection-erosion list.
The single-hit promote-to-MRU placement is settled; spend slots on geometry,
not promotion logic.
