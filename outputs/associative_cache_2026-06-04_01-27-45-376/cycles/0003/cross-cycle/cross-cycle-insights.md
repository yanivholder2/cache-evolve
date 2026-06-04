## Recurring wins

- Leave LRU ordering alone and intervene only on victim selection — the
  one variant that did this (cycle 0001 v003) is still the champion
  after two subsequent cycles of attempted refinement.
- Reuse the existing `Item.lfu_counter` as a binary touched/untouched
  signal: it costs no extra per-item bytes, fits inside the bucket
  trait's current `&[Item]` eviction signature, and degrades
  gracefully when no untouched slot exists.
- "Cheap zero-information fallback" beats clever overrides on this
  trace — the parent's behaviour in the all-touched regime (pure LRU)
  has resisted every attempt to replace it.

## Recurring failures

- Withholding, delaying, or shortening promotion on the access path
  (cycle 0001 v001, cycle 0001 v002 CLOCK, cycle 0002 v003): the
  ~16-slot bucket has no headroom for a hot newcomer to survive its
  vulnerable first-hit window without full promotion.
- Letting a derived counter signal override either raw `lfu_counter
  == 0` or LRU position: counter-rank eviction (cycle 0002 v001/v002),
  on-hit decay (cycle 0003 v001), positional gating (cycle 0003 v002),
  and cardinality-based regime detection (cycle 0003 v003) have all
  regressed hit-rate.
- Treating the counter as anything other than zero/non-zero — without
  any aging mechanism it conflates lifetime hits with recent hits, and
  recency (encoded by bucket position) is the stronger predictor on
  this trace whenever the two disagree.

## Open questions

- Is there an insertion-placement axis (where new items land in the
  bucket) that composes with v003's untouched-first filter, instead of
  competing with it on the eviction-side axis the cohort has now
  exhausted?
- Would widening the bucket trait beyond `&[Item]` — to admit a
  per-slot insertion tick or a bucket-owned reference bit — unlock
  signals that distinguish "fresh insert" from "stale one-shot",
  which every existing-signal refinement has so far failed to do?
- Is cycle 0001 v003 a local optimum of the current trait surface,
  such that further wins require trait widening rather than more
  policy refinement?
- Does the all-touched regime have any detectable proxy (bucket-level
  touched count, absence of any zero slot) that could safely switch
  back to pure LRU — and is "safely" even achievable given how badly
  every regime-aware override has performed?

## Per-policy notes

- ClassicLRU is the only policy explored so far; all variants fork
  from its bucket-level LRU and only one (cycle 0001 v002) tried to
  replace ordering with CLOCK — it lost hit-rate, confirming the
  bucket is too narrow for CLOCK's coarse two-level recency.
- The bucket's small size (~16 slots) is the dominant constraint:
  it kills promotion-side experiments and forces every eviction-side
  signal to be robust in the fully-saturated regime.
- The trait's `&[Item]` eviction signature has been flagged as a
  binding constraint in every cycle's open questions but never
  exercised — it blocks reference bits on the bucket itself and any
  per-slot age state outside `lfu_counter`. This is the most-cited
  unresolved structural thread across the run.
