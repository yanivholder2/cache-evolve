# Cycle 0009 Insight

## Cohort outcome

Three changes were tried: a fifth sketch row at unchanged width (v001), a recency-gated tie-break in admission using a "touched in window" flag (v002), and repacking the sketch as a flat u8 array with single-hash indexing (v003). v002 was named Winner, but all three moved hit-rate only at noise level, so the pick is effectively a tie with the parent.

## Patterns

- Both hit-rate-oriented changes (extra row, tie-break gating) landed within noise. This extends the earlier finding that estimation quality and admission rules are saturated on workload-a.
- The tie-break only matters on equal estimates, and it was nearly neutral. Ties are probably a small share of admission duels.
- v003 was a cost change (size and speed), but its hypothesis text was muddled: it mentioned width 8 while the described change did not use it. Its intended benefits were never measured, because the fitness vectors again report hit-rate only. This is the ninth cycle running.
- This cycle partly repeated earlier ground: sketch depth and admission tuning had already been flat. The hypothesis writer also could not read the overlay or baseline source, so the proposals were generic.
- Variant authors again could not read overlay source, so explanations rest on hypothesis text and fitness vectors.

## Open questions

- Does the flat u8 single-hash sketch actually shrink metadata or speed up ops? It looks behavior-neutral, so it may be a free cost win, but that is unverified.
- Is hit-rate on workload-a exhausted for sketch and admission changes? Eviction or segment structure, or a different trace, may be the only remaining lever.
- Does the v002 flag cost per-entry memory that outweighs its tiny gain?

## Continuation

Do not generate more sketch-depth, tie-break, or admission-rule hypotheses; all are flat. Fix the evaluation first: add metadata-size and per-op-time to the fitness output, plus a second trace (shifting popularity or a different cache size). Then judge the flat u8 packing (v003) and the earlier cost changes (slab links, single-hash) on size and speed. Any width or packing hypothesis must state its parent and scaling direction consistently.
