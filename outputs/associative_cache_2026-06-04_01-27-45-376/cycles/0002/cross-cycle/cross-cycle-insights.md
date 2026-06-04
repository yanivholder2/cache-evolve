# Cross-cycle insights

## Recurring wins

- Eviction-side discrimination over promotion-side gating: leaving LRU
  ordering intact and only biasing victim selection has been the only
  approach to crack the Pareto front so far.
- Reusing the existing `lfu_counter` field as a *binary*
  touched/never-touched signal is effectively free in metadata and
  composes with the bucket trait's `&[Item]` eviction signature
  without churn.
- Sacrificing never-touched slots before genuinely re-used ones is the
  single idiom that has reliably beaten baseline; it remains the
  policy to beat.

## Recurring failures

- Withholding or shortening promotion inside the ~16-slot bucket
  consistently regresses hit-rate — there is no headroom for a hot
  newcomer to survive its vulnerable first-hit window without a full
  move-to-back.
- Replacing exact LRU ordering with a coarser recency proxy (e.g.,
  CLOCK's two-level bit) loses information the bucket was already
  preserving cheaply.
- Pushing `lfu_counter` past the binary touched/untouched distinction
  — whether as tiered filters or smooth counter-rank eviction —
  regresses once every slot is touched, because raw lifetime counts
  start overriding the fresher recency encoded in bucket position.

## Open questions

- Would a cheap aging or decay step (decrement/halve on eviction
  sweep) make multi-bit counter tiering finally outperform the binary
  filter by keeping the counter aligned with *recent* hits?
- Is there a detectable proxy for the "all-touched" regime (bucket-
  level touched-count, absence of any zero slot) that could trigger a
  fallback to pure LRU exactly when counter-based ranking turns
  harmful?
- Would loosening the bucket trait's `&[Item]` eviction signature
  unlock a side-channel reference bit on the bucket itself, opening
  per-op-time wins that the current signature blocks?
- Are there orthogonal axes — insertion placement, bucket-level
  admission, victim-side recency reconstruction — that could compose
  with the untouched-first filter without re-introducing the failure
  modes above?
- Does a "must survive two hits" variant of the filter help in any
  workload regime, or does it always reintroduce the killed-hot-
  newcomer failure mode?

## Per-policy notes

- ClassicLRU: the entire cohort history has revolved around this
  policy. The blanket move-to-back on hit is the primary lever, and
  the bucket size (~16) is the binding constraint on every promotion-
  side experiment. Eviction-side tweaks layered on top of intact LRU
  ordering are the productive direction; promotion-side tweaks and
  ordering replacements are not.
- CLOCK (explored as a promotion-path replacement): coarsening the
  recency signal lost more than it gained inside the narrow bucket;
  would likely need a trait-signature change to host its reference
  bit natively before it is worth revisiting.
- LFU-style counter ranking: only viable as a zero/non-zero gate
  today; full LFU semantics need decay/aging before they can coexist
  with LRU ordering without fighting it.
