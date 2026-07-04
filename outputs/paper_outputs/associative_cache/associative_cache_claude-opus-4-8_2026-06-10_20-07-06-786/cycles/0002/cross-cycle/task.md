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
