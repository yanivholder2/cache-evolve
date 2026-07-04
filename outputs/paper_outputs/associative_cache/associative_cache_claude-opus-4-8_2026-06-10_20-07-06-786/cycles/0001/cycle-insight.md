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
