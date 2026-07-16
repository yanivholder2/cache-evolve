# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0010

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

With every hit-rate lever ruled out over the prior cycles, this cohort stopped
hunting for hit-rate headroom entirely and spent all three slots on
behaviour-preserving cost-side rewrites: a u64 logical-clock timestamp shrinking
`Item` metadata (v001), a `rotate_left` MRU promotion on the get path (v002),
and a rotate-and-replace full-bucket admission on the put path (v003). All three
were provably byte-identical and all three tied on hit-rate, so all three
"won" as undifferentiated ties and the baseline behaviour stands unchanged.

## Patterns

- The whole cohort is a clean demonstration that the ceiling is real: every
  variant was designed to move nothing on hit-rate, and none did. The search
  has fully pivoted from behaviour to implementation cost.
- The three changes target the three distinct cost axes the fitness cannot see
  — metadata footprint (smaller `Item`), get p50 (one suffix memmove instead of
  remove+push with a capacity check), and put p50 (one prefix rotate plus
  in-place replace instead of two memmoves and a length round-trip). Each is
  provably free but only provably *beneficial* on a per-op-time or size axis
  this hit-rate-only vector does not measure.
- This is the same recurring outcome the last three cycles flagged: genuine,
  defensible cost wins keep landing as ties because there is no dimension to
  reward them. The pattern is now systemic, not incidental.

## Open questions

- How long does the loop keep generating tied cost-side variants before the
  lack of a reward axis makes the cohort uninformative?
- The three cost rewrites are independent and stackable — does adopting all of
  them as the baseline change anything the fitness can observe, or do they
  remain collectively invisible?

## Continuation

Hit-rate is exhausted and this cycle adds no behavioural change; adopt v001's
u64 timestamp, v002's rotate-left get promotion, and v003's rotate-replace put
admission as the baseline since all are byte-equivalent and cost-favourable.
The next cycle should not search for hit-rate headroom — there is none left.
The decision that has now been deferred for four straight cycles must be made:
either formally add a per-op-time / metadata-size fitness dimension so these
recurring cost wins become selectable, or declare the trace solved and stop the
loop. Continuing to emit tied cost rewrites under a hit-rate-only fitness is no
longer producing signal.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
