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
