## Recurring wins

- Eviction-neutral layout and lookup changes (sentinel indices, head-hit relink skip, index pre-sizing, hasher swap, lookup collapse) repeatedly held hit-rate equal to their parent on workload-a at size 2048. They are safe to carry forward, but no cycle has yet shown a measured gain from them.
- A bounded frequency bias confined to the LRU end (a small victim window) is the only frequency signal with a reported gain (cycle 0007). It is marginal, and widening it again lost hit-rate (cycle 0009), so it works only when kept narrow.
- Insertion depth is the one lever that has produced a hit-rate gain more than once (tail insertion in cycle 0005, midpoint insertion in cycle 0009). Results for the same idea conflict across cycles, so this is a candidate, not a settled win.

## Recurring failures

- Approximations or structure that change recency handling lowered hit-rate: segmented probation/protected lists (cycle 0005), second-chance referenced bits (cycle 0005), and second-hit promotion (cycle 0006). Strict recency order appears to matter on this trace.
- Frequency counters and admission gates have been flat or slightly down every time they were tried: ghost-set filter (cycle 0007), correlated-reference filter (cycle 0009), decay, saturating counters, and count-min admission (cycle 0008). Frequency signal has not converted into retained hits.
- Widening the frequency-aware victim window beyond four entries lost hit-rate (cycle 0009).
- Winners are often named without a measured effect on their parent (cycles 0001–0009, including cycle 0005's v002, which reported a dip). Selection has not been tied to the fitness vector, so Winner status is not evidence of improvement.
- Infrastructure rejections (cycle 0001, cycle 0004 v002) say nothing about their strategies and should not be counted as failures.

## Open questions

- Per-op time and metadata size have been absent from every fitness vector since cycle 0001. Until both are recorded, no lineage can be judged on the goals that motivated it.
- Does the midpoint insertion marker hold on a second trace or another cache size, and how does its position affect results? Does this reconcile the conflicting tail-insertion outcomes (gain, loss, null)?
- Did the engagement mechanisms actually run: the ghost filter, the count-min gate, the tail-insertion overlay, the midpoint marker, and the correlated-reference filter? Flat results cannot be read as evidence against an idea until engagement is shown.
- Every result so far is on workload-a at size 2048. Generality across traces and sizes is untested.
- Hypothesis labels (H01, H03 in particular) describe different mechanisms in different cycles. Their definitions need to be fixed before future lineages cite them.

## Per-policy notes

- LRU (recency order preserved): layout and lookup changes are hit-rate neutral. No recency-order change has produced a reliable gain; segmentation and second-chance both lost hit-rate.
- LRU with insertion-position changes: tail and midpoint insertion gave mixed results across cycles (gain, loss, null). Treat as unresolved and test depth directly.
- LRU with frequency-aware eviction: bounded near-LRU windows gave at most a slight gain; wider windows lost. Counter, decay, and admission-gate variants were flat or down.
- Ghost-based admission: no engagement evidence yet; the idea remains untested.
