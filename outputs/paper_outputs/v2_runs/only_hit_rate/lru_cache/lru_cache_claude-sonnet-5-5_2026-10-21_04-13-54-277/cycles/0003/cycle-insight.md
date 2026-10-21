# Cycle 0003 Insight

## Cohort outcome

Three variants were tried: a larger protected share for the segmented LRU (90%), a gentler ghost list that re-admits at probation MRU, and a frequency-aware CLOCK (2-bit counter) rewrite of the baseline. Only the protected-share increase (v001) was a Winner. It gave a small hit-rate gain. The ghost variant matched its parent exactly, and the frequency-CLOCK variant showed a modest gain but did not win.

## Patterns

- The segmented line improved by tuning a single constant. Two cycles of evidence now agree: a smaller protected share loses hit-rate and a larger one gains it.
- The gentler ghost policy removed cycle 0002's regression but produced no gain. Its hit-rate was identical to the parent's, so the ghost path likely rarely changed a decision on this trace.
- Frequency-aware CLOCK reports scan-resistance gains similar to the earlier one-bit CLOCK, which suggests that adding frequency bits is incremental, not transformative.
- Every gain this cycle is small and rests on one scan-heavy trace at one cache size.
- Metadata-size and per-op-time were again absent from all fitness vectors. This is the third cycle with that gap, so the claimed size and speed advantages of CLOCK and the cost of the ghost list remain unverified.

## Open questions

- Does the protected-share gain keep growing, plateau, or reverse past 90%? Probation must still absorb scans and give new keys time for a second hit.
- Do the segmented and CLOCK gains hold on other traces and cache sizes, particularly ones with more reuse at long distances where a ghost list might matter?
- Is the missing metadata-size and per-op-time measurement a harness fault? It has now blocked judgment for three cycles.
- Can the protected-share tuning combine with a CLOCK or frequency mechanism?

## Continuation

Resolve the metadata-size and per-op-time measurement gap before any further variants, or the loop is selecting on hit-rate alone. Next, probe the protected share around 90% (for example slightly above, or adaptive sizing) and test the winner on more traces and cache sizes. Drop the probation-MRU ghost list unless a trace with long-distance reuse is added, and stop spending cycles on further single-constant tweaks without a second trace to validate them.
