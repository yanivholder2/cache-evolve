# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants branched from baseline and each lifted hit rate via a
distinct, orthogonal lever: bucket geometry (v001), victim-selection rule
(v002), and insertion position (v003). The Winner, v002, replaced pure-LRU
victim choice with a frequency-first rule by repurposing the dormant
`lfu_counter` as a live hit count.

## Patterns

- Two of the three approaches (v002 victim-by-frequency, v003 probationary
  front admission) attack the same underlying problem — shielding the
  reusable working set from one-hit-wonders and scan traffic — and the
  frequency-first victim rule produced the strongest, cleanest hit-rate win.
- The winning changes stayed entirely intra-bucket: resident set, routing,
  capacity invariants, and the `EvictionPolicyKind` enum / JNI ordinal
  mapping were all left untouched, suggesting victim/admission logic is the
  high-leverage surface this cycle.
- Geometry coarsening (v001) was collision-bound and gave the smallest gain,
  signaling diminishing returns from further widening buckets.
- Every gain came paired with an unmeasured per-op-time or scan cost; the
  fitness vectors reported only hit-rate, so the cost side of these trades is
  currently untracked.

## Open questions

- Is the O(bucket_size) min-frequency scan in v002 affordable, or does its
  per-op cost erode the hit-rate win once measured?
- Do the orthogonal levers compose — e.g. frequency-based victim selection
  combined with probationary front admission — or do they overlap?
- Is the dominant pressure on this workload scan traffic or capacity
  collision? Both signals appeared across variants.

## Continuation

Pursue intra-bucket policy refinements building on v002's frequency signal,
and test whether layering v003's probationary admission on top adds further
scan resistance. Insist on capturing per-op-time alongside hit-rate so the
cost of frequency scans is visible. Drop further geometry coarsening — its
returns are already diminishing.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle stress-tested the prior Winner's frequency-first rule from three
angles: composing it with probationary front admission (v001), aging its
counters under eviction pressure (v002), and a clean-sheet segmented-LRU
built on `lfu_counter` as a promote-on-second-hit flag (v003). Only v003 won;
both modifications to the inherited frequency rule regressed below their parent.

## Patterns

- The two levers from Cycle 0001 do not compose — they conflict. Stacking
  front admission onto frequency-first eviction makes a fresh key
  simultaneously the recency-oldest and minimum-frequency entry, turning it
  into the immediate victim and discarding rising keys at probation.
- The winning scan-resistance came from explicit *segmentation* with a
  promote-on-second-hit rule, not from tuning a single frequency scalar.
  Demanding proof of reuse before granting protection beat both pure-LFU
  victim selection and single-position LRU admission.
- The dominant pressure on this workload is capacity collision, not
  staleness: aging the survivors' counters removed exactly the durable-count
  protection that made high-frequency keys valuable. "Once-hot immortality"
  was a feature here, not a failure mode.
- Both winning and losing variants still pay an O(bucket_size) linear victim
  scan; the per-op cost of that scan remains structurally present but largely
  unmeasured in the reported vectors.

## Open questions

- Can the linear front-scan for the first probationary item be made O(1)
  (e.g. a tracked segment boundary) without losing the SLRU hit-rate gain?
- Is there a probationary/protected sizing ratio that improves on the
  implicit split, or does promote-on-second-hit already capture most of it?
- Since aging hurt and immortality helped, is there any workload regime
  (shifting working set) where staleness *would* dominate — or is this trace
  firmly collision-bound?

## Continuation

Build on v003's segmented bucket as the new baseline: it is the strongest
scan-resistant admission scheme so far. Pursue cost-side refinements —
especially eliminating the linear probationary scan — and segment-sizing
tuning, while keeping hit-rate the primary axis. Drop both counter aging and
front-admission composition; this cycle showed they erode the protection that
durable frequency/segmentation provides on this workload.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle probed the inherited segmented-LRU (v003 of Cycle 0002) along
three axes: probationary eviction order (v002), equal-frequency tiebreak
granularity (v001), and a protected-segment cap with LRU demotion (v003).
The Winner, v002, reversed probation to evict the *oldest* zero-count entry —
making probation a FIFO queue — which recovered rising keys the prior
newest-first churn was discarding.

## Patterns

- Within the segmented bucket, *which* probationary item is evicted is itself
  a high-leverage lever: keeping the same scan but reversing it to oldest-first
  gives a fresh key a full lap to earn its second-hit promotion, and that
  alone lifted hit-rate with no change to admission, promotion, or metadata.
- Two of three changes were inert on hit-rate. The equal-frequency tiebreak
  (v001) and the protected cap (v003) left retention identical — true recency
  and the position proxy pick the same victims, and the protected segment
  never saturated, so its cap-and-demote branch never fired.
- The inert changes split on cost: v001's reorder-free path is a structural
  per-op *saving* (removes the per-hit shuffle), while v003's cap check is a
  structural per-op *cost* (extra linear pass per promotion) with no offset.
- Consistent with prior cycles, the trace remains collision-bound: the gain
  came from giving rising keys a promotion window, not from recency or
  staleness signals, which stayed neutral.

## Open questions

- Is the FIFO-probation win compatible with v001's reorder-free tiebreak — can
  the per-op saving be banked alongside the hit-rate gain?
- Does any workload regime ever saturate the protected segment, making v003's
  cap relevant, or is it permanently dead code on this trace?
- With probation order now tuned, what is the next hit-rate lever — segment
  sizing, or is the FIFO+promote-on-second-hit scheme near its ceiling here?

## Continuation

Adopt v002's oldest-first (FIFO) probation as the new baseline — it is the
strongest scan-resistant scheme so far. Pursue composing it with v001's
reorder-free tiebreak to capture per-op savings without risking hit-rate, and
push on segment-sizing as the remaining hit-rate axis. Drop the protected-cap
demotion: it is inert on this trace and only adds promotion-path scan cost.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle attacked the inherited FIFO-probation segmented-LRU from three
directions — a stricter two-hit promotion threshold (v001), a capped
protected segment with demotion-to-probation (v002), and a CLOCK
reference-bit substitute for recency ordering (v003). None beat the parent:
all three regressed hit-rate. v001 was carried as the Winner only as the
least-harmful change; the prior FIFO baseline effectively stands unimproved.

## Patterns

- Every variant lost by *withholding or stripping protection* from keys the
  parent kept. Raising the promotion bar (v001) stranded once-hit keys in
  FIFO until eviction; capping the protected segment (v002) demoted proven
  hot keys back into harm's way; CLOCK (v003) discarded the fine recency
  ranking that shielded warm keys. The trace punishes any scheme that makes
  earned protection harder to get or easier to lose.
- This confirms a multi-cycle boundary: one probationary hit is sufficient
  proof of reuse here, and the protected segment is load-bearing and does
  not want shrinking. Both the second-hit threshold (this cycle) and counter
  aging / front-admission (earlier cycles) failed for the same reason —
  they erode durable protection on a collision-bound trace.
- Recency granularity matters: a coarse one-bit reprieve underperforms full
  positional ranking in small buckets, so recency ordering is not safely
  approximated away.

## Open questions

- With promotion threshold, protected cap, and recency approximation all
  ruled out, is the FIFO-probation + promote-on-first-hit scheme at or near
  its hit-rate ceiling on this trace?
- Is there a *looser* protected sizing (larger, not capped) that helps, or
  is the current implicit split already optimal?
- Are the remaining gains now cost-side only (banking the per-op savings
  from reorder-free tiebreak) rather than hit-rate?

## Continuation

Keep Cycle 0003's FIFO-probation segmented-LRU as the baseline; this cycle
produced no hit-rate improvement on it. Stop probing protection-tightening
levers — second-hit thresholds, protected caps, and recency approximations
have now each failed for the same protection-erosion reason. Redirect the
next cycle toward (a) cost-side wins such as composing the reorder-free
tiebreak to bank per-op savings, and (b) untested hit-rate axes like looser
protected sizing or bucket geometry interactions, since the admission and
promotion logic appears saturated.


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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
