# Cycle 0006 Insight

## Cohort outcome

Three variants were tried. Two changed main-queue eviction: a bounded lookahead that picks the lowest-freq entry (v001), and a cap on reinsertions per eviction (v002). The third (v003) combined known admission settings: main evictees go to the ghost, a very small small queue, and a shorter ghost. Only v003 won, with a hit-rate gain. Both eviction variants lost a little.

## Patterns

- Every change to main-eviction mechanics lost hit-rate. Replacing reinsertion with lookahead lost, and capping reinsertion depth also lost. The unbounded second-chance reinsertion scan is doing real protective work.
- The one Winner is a consolidation of earlier admission gains (small queue shrink, shorter ghost), plus ghost insertion of main evictees. It adds no per-key state. This again points to admission sizing, not eviction, as the lever that moves this trace.
- Ageing freq in place for skipped entries (v001) fits the older finding that ageing and decay changes lose here.
- Cost was still unmeasured. Variant authors still could not read overlay source. Evidence is one trace at one cache size, so small regressions may be noise.

## Open questions

- Which part of v003 carries the gain: ghost insertion of main evictees, or the re-bundled sizing?
- Does v003's extra ghost work on `put` cost meaningful per-op time?
- Does the gain hold on a second trace or cache size?

## Continuation

Adopt v003 as the base. Drop main-eviction rewrites (lookahead, reinsertion caps, freq ageing) and keep the parent's reinsertion semantics. Next, isolate whether ghost insertion of main evictees helps, and validate on a second trace and cache size. Restore metadata-size and per-op-time measurements so cost can be compared.
