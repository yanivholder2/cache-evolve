# Cycle 0002 Insight

## Cohort outcome

The cohort tried one representation-level change (replacing optional links with a sentinel index) and two behaviour-level ideas: a ghost-based admission filter and pre-sized index allocation. Only v001 produced a usable result and was named the Winner, and its measured gain is on hit-rate, not on the node-size goal its change was written for.

## Patterns

- The Winner's change is a layout change that should preserve eviction order, and its hit-rate did not move from its parent. That is consistent with behaviour preservation, but it says nothing measured about metadata size.
- The same sentinel-index change already appears as cycle 0001's v003 credit, so the size claim is still carried forward without a measurement.
- The hypotheses differ in how much they touch eviction. H01 is a layout change that must keep hit-rate fixed; H02 deliberately changes what stays resident and must be judged on hit-rate and per-op cost; H03 leaves policy and layout alone and targets per-op time only.
- Per-op time has still never been measured on any lineage, so no cohort has yet produced evidence on that dimension.

## Open questions

- Lineage is ambiguous: the hypotheses describe v001's hit-rate as a gain over baseline, while the variant insight reports no movement against its parent. Which parent the gain is measured against needs to be pinned down before any comparison.
- H01 proposes re-applying a sentinel change that v001 already appears to contain. Is there a remaining size effect to find, or is H01 redundant?
- Does ghost-based admission (H02) improve hit-rate enough to justify its bookkeeping cost on the per-op dimension?

## Continuation

Next cycle should prioritise measuring per-op time and metadata size on v001 and the existing sentinel lineage, since neither has real evidence yet. Pursue H03 as a cheap, policy-neutral per-op check against the baseline, and treat H02 as the only hit-rate-changing bet worth running. Drop H01 unless it is reframed to measure size on the current winner rather than re-applying the same change.
