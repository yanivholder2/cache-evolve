# Cycle 0003 Insight

## Cohort outcome

Two variants made the newcomer filter more aggressive: v001 moved insertion depth from 1/4 to 1/8, and v003 left the GCLOCK hand resting on the newcomer it just inserted, which gives a grace window of about one miss. v002 instead added frequency-aware victim choice within the bottom 4 slots of the 1/4-depth line. v001 won. v003 passed the previous cycle's depth winner and got close to v001, while v002's gain was barely above noise.

## Patterns

- For the third cycle running, how fast a newcomer must prove reuse is the main lever on workload-a. Both variants that strengthened the newcomer filter improved, from two different designs: shift-based LRU and in-place CLOCK. A filter as short as about 1–2 misses still did not hurt hit rate on this trace.
- Returns are shrinking along the depth axis: the step from 1/4 to 1/8 gained less than the step before it. Together with v003's near-LIP result, this suggests hit rate on workload-a is flattening near the eviction end.
- The CLOCK line's remaining gap was mostly about where the hand rests, not about frequency. Once the newcomer filter matched, in-place GCLOCK came close to deep-insertion LRU. Its no-shift design may make it the better choice once latency is counted.
- Frequency signal at the eviction end adds almost nothing when hits promote to MRU. This is the second cycle in which frequency-based or extra protection beyond the newcomer filter brought little or no hit-rate gain.
- Per-op-time and metadata-size are still not recorded, three cycles in. Every latency claim is still untested: v001's larger shifts, v002's window scan, and v003's expected O(1) miss path.

## Open questions

- Does pure LIP (index 0) help, or has v001's curve already reached its peak?
- Is the one-miss grace window over-fitted to workload-a? Traces with longer reuse distances or other cache sizes could penalise both v001 and v003.
- Once latency is measured, does v003's in-place design beat v001 overall despite a slightly lower hit rate?

## Continuation

Finish the depth bracket with pure LIP on v001, and test both v001 and v003 on at least one other trace or size to check whether the aggressive filter generalises. Make per-op-time recording a precondition. Without it, the cohort cannot be ranked between v001 and v003. Drop victim selection by frequency at the eviction end. If frequency is revisited, apply it on the promotion or insertion side, for example by setting insert depth from a key's count.
