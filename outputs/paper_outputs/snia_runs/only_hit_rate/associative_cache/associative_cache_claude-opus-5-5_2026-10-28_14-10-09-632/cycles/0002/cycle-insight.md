# Cycle 0002 Insight

## Cohort outcome

Two variants extended cycle 0001's cold-insertion winner (v001). v001 added an absolute probation floor. v002 added BIP-style occasional MRU insertion. The third variant, v003, upgraded CLOCK to a 2-bit GCLOCK counter. The Winner is v002. Its overall gain was small and came with a split by trace: it helped hm_0, mostly at the small size, and hurt web_0.

## Patterns

- Both v001-lineage variants went after the same small-bucket shortfall, and both recovered some of it. hm_0 benefited more than web_0 each time, so hm_0 has meaningful reuse beyond the original probation window at 16384.
- Gains at the small size don't carry over to the large size. Lengthening probation left 1024-slot buckets unchanged. Sending items to MRU hurt them on web_0, where cold traffic is mostly scan-like and already drained well. A single fixed rule can't serve both trace types.
- The frequency-aware CLOCK line (v003) improved on its parent in every trace×size cell, especially on hm_0 at the large size, and it does no element shifting. It was not the Winner, but it is now a credible second lineage. Its advantage grows with bucket size, where the v001 line has the least room left to improve.
- For the second cycle in a row, no variant measured per-op-time or metadata-size. Every cost claim is still untested: mid-vector inserts, GCLOCK's multi-rotation sweeps, and the full-bucket candidate scan.

## Open questions

- Would adaptive insertion fix v002's trace split? For example, DIP-style set dueling between LIP and BIP, or an MRU-insert rate that scales with bucket size.
- Do the absolute floor (v001) and occasional MRU insert (v002) stack, or do they recover the same reuse?
- Does GCLOCK's hit-rate gain hold once its worst-case sweep cost is measured? And can it take a cold-insertion analogue, such as starting new items below already-referenced ones?

## Continuation

Build on v002, but replace its fixed period with an adaptive policy (set dueling, or a rate tied to bucket size and trace behavior) so web_0 stops losing hits. Try combining it with v001's probation floor. Keep the GCLOCK lineage (cycle 0002 v003) going as a parallel branch, since it scales well to large buckets. Make per-op-time measurement a requirement, not an option. Drop further tuning of the BIP period alone.
