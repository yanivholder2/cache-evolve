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
