## Recurring wins

- Eviction-preserving representation and hit-path changes (sentinel indices, head-hit relink skip, index pre-sizing, hit-path lookup collapse) have repeatedly kept hit-rate equal to their parent across cycles 0001–0004. This shows they are behaviour-safe, not that they help.
- The same sentinel-index change is credited in both cycle 0001 and cycle 0002, so its size benefit has been carried forward twice without a measurement.

## Recurring failures

- Winners are being named on evidence outside their stated goal. Metadata size and per-op time were the targets in cycles 0001–0003, yet no Winner was scored on either. Winner status has rested on hit-rate parity alone (cycle 0003 v002 had no reported gain at all).
- The ghost-based admission filter (cycles 0002 and 0003) has produced no measured movement. It may never have engaged on the trace, so its null result is uninformative.
- Rejections have been infrastructure-driven, not strategy-driven (session limit in 0001, missing insight files in 0001 and 0004). Those approaches have no evidence either way.
- The selection criterion is unclear. A variant with the only reported hit-rate movement (cycle 0004 v003) was not chosen as a Winner over a parity-only variant.

## Open questions

- Per-op time and metadata size have not been measured on any lineage. Until they are, no claim about the sentinel, head-hit, pre-sizing, or hit-path changes can be confirmed.
- Which parent is the reference for hit-rate comparisons (raised in cycle 0002, still unresolved)?
- What is the direction and size of the segmentation variant's hit-rate effect (cycle 0004 v003), and does it carry a per-op or size cost from its extra list pointer and demotion logic?
- Is the H03 mapping for the segmentation variant correct? Earlier cycles framed H03 as a policy-neutral per-op check, not a segmentation change.
- Does the ghost-based admission filter ever engage on the current trace, and if not, which workload would exercise it?

## Per-policy notes

- LRU (recency list, layout and hit-path changes): eviction-preserving edits hold hit-rate at parent level. The open gap is whether they reduce size or per-op time, which has not been measured.
- Segmented LRU (probation/protected split): the only candidate so far that moves hit-rate, in an unknown direction. It likely adds per-op and size cost, which is unmeasured.
- Admission filtering (ghost-based): no measured effect so far. Engagement on the trace is unverified, so this should not be judged a null until that is checked.
