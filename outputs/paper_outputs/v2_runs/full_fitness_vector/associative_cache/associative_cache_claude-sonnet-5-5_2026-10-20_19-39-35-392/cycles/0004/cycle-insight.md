# Cycle 0004 Insight

## Cohort outcome

Variants targeted cost reduction around the mid-insertion/transpose baseline: a per-bucket ghost admission filter (v001), O(1) swap-based mid-insertion (v002), and single-hash/mask indexing with tag-first scan (v003). v002 was a clear Winner and v001 the other Winner on thin evidence, since its own insight reports a hit-rate loss and slower puts. v003 sped up put only.

## Patterns

- Put-path cost was the lever that moved. Removing the half-bucket shift (v002) and redundant hashing/modulo (v003) both cut put latency, with hit rate and metadata unchanged.
- Approximate ordering among displaced entries does not matter. Where the new entry lands matters, not the exact order of the rest, so exact bucket ordering is not worth paying for.
- A one-entry-per-bucket ghost is too weak a signal. It added put work and slightly hurt hit rate, penalizing reusable keys along with scan items.
- Get latency did not improve through any real mechanism. This is the fourth cycle with no get gain. Arithmetic, hashing and item movement are now all ruled out, and small get shifts look like noise or codegen effects.
- No variant grew metadata.

## Open questions

- Is the v001 Winner status real, or within noise given the negative insight?
- Is get time dominated by memory access to the bucket (cache lines, slot layout, tag/key separation)? It is untested.
- Would a deeper or more selective admission history help, or is scan resistance already saturated by insertion position?
- Do any gains hold on a workload other than workload-a?

## Continuation

Adopt v002's swap-based mid-insertion as the policy baseline and fold in v003's hash-once indexing only if it can be made to help rather than hurt get. Drop single-entry ghost filters and further arithmetic or hash micro-optimizations. Aim the next hypotheses at memory layout for get (cache-line alignment, packing tags or keys contiguously, prefetch) and validate on a second workload.
