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
