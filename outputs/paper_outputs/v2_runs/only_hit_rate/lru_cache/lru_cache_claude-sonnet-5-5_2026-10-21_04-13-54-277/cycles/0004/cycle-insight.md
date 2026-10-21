# Cycle 0004 Insight

## Cohort outcome

Three variants refined the segmented LRU: a larger protected share (95%), a second-chance bit on protected eviction, and a stamp-based fast path that skips relinking on hits. Only the protected-share increase (v001) won, with a marginal hit-rate gain. The second-chance variant matched its parent, and the relink-skip variant lost a little hit-rate.

## Patterns

- The protected-share curve is nearly flat past 90%. The gain is marginal, and probation is close to its minimum useful size.
- Refinements to protected-segment eviction or recency tracking did not help. The second-chance bit sits on a fallback path that rarely runs at this share. The approximate-recency shortcut gave up some hit-rate.
- Cheaper-per-op ideas (skipping relinks) cannot be judged. Metadata-size and per-op-time are absent from every fitness vector for the fourth cycle in a row.
- Gains are again small and rest on one scan-heavy trace at one cache size.

## Open questions

- Is the measurement gap for metadata-size and per-op-time a harness fault? It has now blocked judgment for four cycles.
- Do the segmented gains hold on other traces and cache sizes, especially ones with long-distance reuse or less scan pressure?
- Would adaptive segment sizing (ARC-style) beat any fixed share when the workload changes?
- Does the relink-skip shortcut pay for itself in time, if per-op-time were measured?

## Continuation

Stop tuning the protected share and drop protected-eviction refinements such as the second-chance bit, because both are at a plateau on this trace. Before any new variants, fix the metadata-size and per-op-time measurement and add at least one more trace or cache size. After that, pursue changes that act on probation or promotion, such as adaptive sizing or combining the segmented design with CLOCK or frequency ideas. Revisit the relink-skip only if per-op-time can show its benefit.
