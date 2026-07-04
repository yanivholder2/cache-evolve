# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0008

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants built on the FIFO-probation segmented-LRU baseline and
attacked it from different angles: an LFU-min fallback for full-protection
eviction (v001), a ghost-ring accelerating re-entry of recently-evicted keys
straight into the protected segment (v002), and a cost-side fast path that
elides a no-op reorder on the hottest hit path (v003). v003 won — but only as
a hit-rate-neutral tie; both behavioral changes regressed below the parent,
so the baseline again stands hit-rate-unimproved.

## Patterns

- The two hit-rate attempts failed for the same reason that has recurred all
  run: both weakened the discriminating power of earned protection. The LFU
  fallback let `== 1` newcomers dominate the tally and killed freshly-promoted
  rising keys; the ghost ring let weakly-warranted bounce-back keys seize
  protected space without earning it. Recency at the protected end and
  earn-it-each-time probation both beat their proposed replacements.
- This sharpens the multi-cycle boundary: on this collision-bound trace,
  "recently evicted" is not a reliable proxy for "hot," and banked frequency
  counts do not separate hot from rising keys better than plain protected-LRU
  recency. The admission/promotion/eviction logic is now repeatedly confirmed
  saturated for hit-rate.
- The only non-regressing change was purely cost-side, and it landed as an
  undifferentiated tie because hit-rate is the sole fitness axis. A provably
  behavior-neutral per-op saving is invisible to selection — there is no
  dimension to reward it.

## Open questions

- Is hit-rate now effectively at its ceiling on this trace, given that
  admission, promotion, protected-eviction signal, and re-entry have all been
  ruled out across cycles?
- Without a per-op-time fitness dimension, can any cost-side refinement ever
  be selected for, or will it always tie? Is adding such an axis warranted?
- The one untested hit-rate axis remaining is looser/larger protected sizing
  and bucket geometry — does enlarging protection (rather than tightening or
  re-routing it) move the needle?

## Continuation

Keep the FIFO-probation segmented-LRU as the baseline; this cycle produced no
hit-rate gain. Stop probing protected-eviction re-signaling (LFU fallback) and
accelerated re-entry (ghost rings) — both join the long list of
protection-erosion failures. The behavioral logic looks saturated for
hit-rate. Redirect the next cycle to the two genuinely untested directions:
(a) looser/larger protected sizing and bucket-geometry interactions as the
last plausible hit-rate lever, and (b) recognizing that cost-side wins like
v003 are real but unrewardable under a hit-rate-only fitness — consider whether
a per-op-time axis is needed before spending more cohort slots on them.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
