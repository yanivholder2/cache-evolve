# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants attacked ClassicLRU's blanket move-to-back behaviour
inside the bucket and reused the existing `Item.lfu_counter` field, so
metadata size never grew. The Pareto front is held by v003, which kept
LRU ordering intact and only added an eviction-side filter that
sacrifices never-touched slots before genuinely re-used ones.

## Patterns

- The two strategies that touched the access path (v001 delayed
  promotion, v002 removed reordering entirely for CLOCK) both regressed
  hit rate; the one strategy that left promotion alone and changed only
  victim *selection* won.
- The bucket (~16 slots) is too narrow for scan-resistance via
  withheld promotion to pay off — there is no headroom for a hot
  newcomer to survive its vulnerable first-hit window if it isn't
  promoted, and CLOCK's coarse two-level recency loses information that
  exact `remove`+`push` was preserving.
- Reusing `lfu_counter` as a one-bit "has been touched" signal is
  effectively free: it costs no per-item bytes and the bucket trait's
  `&[Item]` eviction signature accommodates it without changes.

## Open questions

- Does a deeper counter (2-bit aging instead of binary touched/untouched)
  extend v003's churn-absorbing effect, or does the bucket's small size
  make further gradations noise?
- The trait's `&[Item]` eviction signature forced v002's side-channel
  design; would loosening it unlock cheaper per-op-time wins without
  the hit-rate regression CLOCK suffered here?
- v003 protects after one hit — is there a workload regime where a
  "must survive two hits" filter helps further, or does it reintroduce
  the v001 failure mode of killing hot newcomers?

## Continuation

Pursue eviction-side discrimination, not promotion-side gating: the
winning idiom is "keep LRU ordering, bias the victim pick." Future
hypotheses should explore richer cheap signals on top of the existing
`lfu_counter` (multi-bit aging, decay on eviction sweep) and consider
hybrids that combine v003's filter with insertion-placement tweaks.
Drop further variations on H01-style "withhold promotion" and on H02-
style "replace ordering with CLOCK" unless paired with a trait-signature
change that lets the reference bit live on `Item` itself.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants tried to refine cycle 0001's winner along the two
axes it had highlighted — finer eviction-side counter tiering (v001),
continuous counter-rank eviction (v002), and a smaller promotion-side
tweak (v003 of this cycle) — and every one of them regressed hit-rate
versus the prior champion. Only v001 reached the Pareto front, and only
because the cohort failed to improve hit-rate at all while staying flat
on metadata size.

## Patterns

- The all-touched regime is where every variant broke. Once every slot
  has `lfu_counter > 0`, both a binary "skip ones" filter (v001) and a
  smooth min-counter rank (v002) start fighting recency: counter value
  rewards hits earned arbitrarily far in the past, while the back-of-
  bucket position still encodes the freshest re-references. On this
  trace recency was the stronger predictor in that regime, so any
  policy that lets the counter override it loses.
- Raw `lfu_counter`, without any aging or decay, is not a drop-in
  refinement signal on top of LRU ordering. Cycle 0001's v003 succeeded
  because it only consulted the counter to distinguish *never-touched*
  from *touched*; pushing the signal further demands either decay or a
  regime-aware fallback to pure LRU.
- Shortening promotion *distance* (this cycle's v003) failed the same
  way cycle 0001's v001 failed by withholding promotion entirely: a
  one-slot swap doesn't give a hot newcomer enough headroom against
  bucket insert pressure between its first and second hit. The
  promotion-side axis remains hostile inside a ~16-slot bucket.

## Open questions

- Would adding a cheap aging step — e.g., decrementing or halving every
  counter during an eviction sweep — let multi-bit counter tiering
  finally outperform v003's binary filter, by ensuring the counter
  reflects *recent* hits rather than lifetime hits?
- Is there a detectable proxy for the "all-touched" regime (e.g., a
  bucket-level touched-count, or absence of any zero slot) that could
  switch the policy back to pure LRU exactly when counter-based ranking
  becomes harmful?
- The trait's `&[Item]` eviction signature still blocks side-channel
  reference bits on the bucket itself; the cohort never exercised this,
  but cycle 0001 already flagged it as a constrained axis.

## Continuation

Cycle 0001's v003 is still the policy to beat, and this cycle's
negative results sharpen *why*: the counter is only trustworthy as a
zero/non-zero signal, and the promotion path inside a small bucket has
no slack to give up. Next cycle should drop further variations on
counter-rank eviction (H01 and H02 patterns) and on shortened-promotion
access paths (this cycle's H03 pattern) unless paired with a real
decay mechanism or a regime switch. Prefer hypotheses that either
(a) add aging/decay so counter rank stops conflating stale hits with
fresh ones, (b) compose v003's untouched-first filter with an
orthogonal axis the cohort has not yet touched — insertion placement,
bucket-level admission, or victim-side recency reconstruction — or
(c) explicitly fall back to pure LRU once the bucket is fully touched.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants forked cycle/0001/v003 and attacked exactly the three
axes the prior cycle's open questions named — on-hit decay (v001),
positional gating of the untouched-first scan (v002), and a cardinality
proxy for the all-touched regime (v003) — and every one of them
regressed hit-rate against the parent. Only v003 reached the Pareto
front, and only by being the least costly regression while staying flat
elsewhere.

## Patterns

- The intuitive readings of each signal all inverted on this trace.
  Decay was supposed to convert lifetime hits into recent-relative
  protection but instead drained genuine re-referenced items back to 0.
  Back-half position was supposed to mark "hot newcomer" but in
  practice marked "freshly-inserted one-shot." A lone untouched slot
  was supposed to be the freshly-inserted keeper but in churn-heavy
  buckets is just as often a stale one-shot the parent's scan was
  correctly killing.
- v003's untouched-first scan is robust because it commits to a binary
  signal and degrades gracefully when no untouched slot exists. Every
  refinement this cycle tried — temporal (decay), spatial (front-half),
  or population-based (count >= 2) — replaced correct evictions in
  some sub-regime with incorrect ones, with no offsetting gain in the
  regimes those refinements were aimed at.
- The cohort confirms cycle 0002's diagnosis from a new angle: any
  policy that lets a derived signal override either the raw
  `lfu_counter == 0` predicate or LRU position loses on this workload.
  The headroom inside a ~16-slot bucket is too narrow for the parent's
  zero-information fallback to be the wrong default.

## Open questions

- Are there signals outside `lfu_counter` + bucket position — e.g., a
  per-slot insertion tick, or bucket-level admission state — that could
  distinguish "fresh insert" from "stale one-shot" without the failure
  modes this cycle hit? The cohort never reached this axis because the
  trait's `&[Item]` eviction signature still constrains side-channel
  state on the bucket.
- Have we exhausted the eviction-side axis entirely on this trace, or
  does composing v003's filter with an orthogonal *insertion-placement*
  axis (where new items land in the bucket, not which slot is evicted)
  still have headroom?
- Is cycle/0001/v003 close to a local optimum within the current trait
  surface, such that further wins require widening the trait rather
  than refining the policy?

## Continuation

Cycle/0001/v003 has now survived two cycles of attempted refinement
along every axis the prior digests flagged as promising. Drop further
variations on on-hit decay, positional gating of the eviction scan, and
cardinality-based regime detection — this cohort independently
falsified each of the three open questions cycle 0002 left open. Next
cycle should either (a) move off the eviction-side axis entirely and
explore insertion placement or per-slot age signals that the existing
trait can carry, or (b) explicitly widen the bucket trait so genuinely
new state (insertion tick, reference bit on the bucket itself) becomes
expressible — every refinement of the existing signal space has now
underperformed the parent.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants attacked cycle/0001/v003 along axes the prior digest
flagged as still-unexercised — insertion placement (v001), cursor-style
fairness among the filter's candidates (v002), and reversed scan
direction (v003) — and every one regressed hit-rate against the
parent. v002 reached the Pareto front only by spending the regression
on per-bucket cursor state that kept `Item` metadata flat.

## Patterns

- Each variant independently revealed that v003's apparent defaults
  were doing load-bearing work. Push-to-back insertion was the grace
  period that let a new key survive long enough to bump its counter.
  Leftmost-first scan was a free age signal under move-to-back
  dynamics, with the oldest never-hit slot landing at index 0. Both
  the placement axis and the scan-direction axis turn out to be
  *coupled* to the `lfu_counter == 0` filter through ClassicLRU's
  move-to-back, not independent of it.
- The cycle 0003 v002 autopsy claim that back-of-bucket untouched
  slots are predominantly one-shot churn was falsified twice in one
  cohort — once by v003's direct test (reverse scan) and once by
  v001's indirect test (gap-fill landing fresh items mid-bucket).
  On this trace those slots are mostly fresh arrivals waiting for
  their first hit, not stale one-shots.
- Spreading eviction pressure for fairness (v002) destroyed information
  rather than refining the parent: when the existing index ordering
  already correlates with a useful signal, CLOCK-style rotation strips
  that correlation without supplying a replacement.

## Open questions

- Are there *any* axes the existing trait surface still exposes that
  aren't coupled through ClassicLRU's move-to-back dynamics back into
  v003's filter? Every "orthogonal" axis the prior digests named has
  now turned out to share the same hidden coupling.
- Does the cohort's failure to find headroom anywhere on the current
  trait surface make it time to widen the `Bucket`/`EvictionPolicy`
  trait so genuinely new state (insertion tick, bucket-side reference
  bit) becomes expressible — the move cycle 0003 already flagged?
- Is per-bucket state that doesn't add per-item bytes (v002's cursor
  pattern) a vehicle worth keeping for *other* signals, even though
  the rotation use of it lost?

## Continuation

Cycle/0001/v003 has now survived three cycles of refinement. Every
axis previously flagged as orthogonal — counter aging, positional
gating, regime detection, insertion placement, scan direction,
cursor-based fairness — has been tried and lost, with the failures
all explained by the same root cause: ClassicLRU's move-to-back
dynamics make the bucket's slot ordering itself a signal that v003
already consumes. Drop further attempts to refine the policy within
the current trait surface. Next cycle should either (a) widen the
`Bucket` or `EvictionPolicy` trait so new per-bucket state (insertion
tick, reference bit) becomes expressible without piggybacking on
slot position, or (b) explicitly accept v003 as a local optimum on
this trait surface and shift attention to the metadata-size axis,
since hit-rate appears saturated for the signals currently available.


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
