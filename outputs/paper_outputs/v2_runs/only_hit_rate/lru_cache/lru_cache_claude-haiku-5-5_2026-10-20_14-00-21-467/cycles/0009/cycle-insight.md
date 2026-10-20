# Cycle 0009 Insight

## Cohort outcome

The cohort tried widening the frequency-aware tail scan (v001), a correlated-reference filter on counted hits (v002, the Winner), and linking new keys at a mid-recency marker instead of the MRU head (v003). The only hit-rate gain reported this cycle belongs to v003, which is not the Winner; the Winner v002 shows no measurable change against its parent.

## Patterns

- Insertion depth is the only lever this cycle with a reported hit-rate gain. Placing fills partway down the recency order (v003) reduced displacement of the hot head on workload-a at size 2048. Its insight does not confirm the marker was engaged, so the gain is consistent with the mechanism but not yet attributed to it.
- Earlier cycles tested tail insertion with conflicting results (gain in 0005, loss in 0006, null in 0007). The midpoint form is a different position from MRU-tail insertion, so the conflict may be about insertion depth rather than the idea as a whole. This is a hypothesis to test, not a finding.
- Widening the frequency-aware victim window past four entries (v001) lost hit-rate. This matches the bounded-window result in cycle 0007: frequency bias helps only near the LRU end, and deeper scans evict entries the trace re-references.
- The Winner v002 has no recorded gain, and its own insight cannot confirm the filter landed. The selection-criterion gap flagged since cycle 0003 recurs: the only measured improvement sits in a non-Winner.
- Metadata size and per-op time are again absent from every fitness vector. This is the same gap carried since cycle 0001.

## Open questions

- Does the midpoint-insertion gain (v003) hold on a second trace or another cache size, and does it change when the marker sits at other depths?
- Did the v003 marker actually receive insertions, and did the v002 correlated-reference filter ever reject a hit?
- What does the midpoint marker cost in metadata and per-op time?

## Continuation

Pursue insertion depth as the lead lineage: build on v003's midpoint marker and test a small sweep of marker positions on a second trace before stacking anything on it. Drop widening the frequency-aware tail window beyond four (v001). Do not extend v002 as the Winner until it records a measured effect; verify its filter engaged before ruling the idea in or out.
