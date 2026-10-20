# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0009

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cohort probed the new single-halving geometry from three sides: a
second capacity-conserving widening step (v001), an absolute associativity
target replacing relative halving (v003), and a behaviour-preserving O(1)
rewrite of the eviction-victim scan (v002). Both geometry moves regressed,
pinning the sweet spot at exactly one halving; v002 won only as a hit-rate
tie — a provably free cost-side change with no behavioural effect.

## Patterns

- The geometry axis that opened up last cycle is now bracketed and closed.
  A second halving (≈4× associativity) overshoots — too few buckets scatter
  hot keys into crowded sets and lose more to routing than wider buckets
  recover. The single halving is confirmed optimal, with the routing-loss
  boundary sitting right at the second step.
- The two regressions failed by the same width mechanism from opposite
  directions: v001 made buckets too wide, v003's absolute target degenerated
  into a fixed 8-way split *narrower* than the parent. On a power-of-two
  budget the absolute scheme buys no resolution the relative halving lacks —
  it just hard-codes a sub-optimal width. Both confirm the parent's width is
  the live setpoint, not a way station to a finer one.
- The lone non-regressing change was, once again, purely cost-side and landed
  as an undifferentiated tie. v002 reclaims the now-costlier per-eviction scan
  the wider buckets introduced, but that saving lives on a per-op-time axis
  the hit-rate-only fitness cannot see or reward.

## Open questions

- With geometry now bracketed on both sides, is hit-rate fully at its ceiling
  on this trace — admission, promotion, eviction, re-entry, and now bucket
  width all exhausted?
- Does any interaction remain between the settled single-halving width and
  segment sizing, or are the two axes confirmed independent and both pinned?
- The wider buckets made per-op scan cost real; v002 banks it invisibly. Is a
  per-op-time fitness axis now warranted so these recurring cost-side wins can
  be selected rather than perpetually tie?

## Continuation

Keep the single-halving FIFO-probation segmented-LRU as the baseline; v002's
O(1) eviction rewrite is behaviour-identical and safe to adopt for its cost
profile but adds no hit-rate. Stop probing bucket width — it is now bracketed
above (second halving) and below (8-way absolute target), and one halving is
settled. The behavioural logic remains saturated from prior cycles. With every
known hit-rate lever now exhausted, the next cycle should either confront the
ceiling directly (look for an untested trace-structure axis) or formally weigh
adding a per-op-time fitness dimension, since cost-side wins keep recurring and
keep tying.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cohort confronted the declared ceiling head-on by testing the three
last-untested hit-rate axes: routing quality via an avalanche finalizer plus
bias-free reduction (v001), an additive "elite" third protected tier (v002),
and a non-eroding reserved-probationary cap at `bucket_size - 1` (v003). All
three regressed; v001 won only as the least-bad regression, so the
single-halving FIFO-probation segmented-LRU baseline again stands unimproved.

## Patterns

- The routing axis is now ruled out. `DefaultHasher` (SipHash) is already
  effectively uniform on this trace's keys, so a stronger finalizer and
  Lemire reduction only reshuffle which keys collide — no headroom to exploit,
  small net loss. The ceiling is set by bucket policy and capacity, not by
  uneven key distribution.
- Both protection-side moves failed against the same "once-hot immortality"
  property the trace keeps rewarding. The elite tier (v002) sacrificed
  merely-once-hot keys that are still re-referenced, and it broke the parent's
  clean recency ordering by evicting on tier before recency. The reserved
  probationary slot (v003) traded one immortal protected slot for an
  un-promotable churning slot, costing more retained-hot capacity than it
  saved from newcomer churn.
- This resolves two long-standing open questions negatively: discriminating
  *within* the protected set does not help, and the "looser/larger protected
  sizing" lever — even in its non-eroding, no-demotion form — slightly lowers
  hit-rate. Full immortality at the current protected size is the right policy.

## Open questions

- With routing, protected-tiering, and protected-sizing now all closed
  alongside the previously bracketed geometry and saturated behavioural logic,
  is any hit-rate lever left, or is the ceiling fully reached on this trace?
- Cost-side wins (the O(1) eviction rewrite, the elided reorder) keep recurring
  and keep tying — is a per-op-time fitness axis now warranted to make them
  selectable rather than invisible?

## Continuation

Keep the single-halving FIFO-probation segmented-LRU baseline; this cycle
produced no hit-rate gain. Stop probing routing quality (SipHash is already
uniform), within-protection tiering (elite tier net-negative), and protected
sizing/reservation (negative in every form tested) — these join geometry width
and behavioural admission/promotion/eviction on the exhausted list. Essentially
every structural and behavioural hit-rate lever is now ruled out. The next
cycle should stop spending slots searching for hit-rate headroom and instead
formally weigh adding a per-op-time fitness dimension, since that is the only
axis where the recurring cost-side wins can actually be rewarded.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
