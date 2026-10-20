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
