## Recurring wins

- Eviction-neutral changes (sentinel indexing, relink skipping, index pre-sizing) have consistently kept hit-rate equal to their parent across cycles 1 to 3. This fits behaviour preservation, but it is not yet a measured gain.
- The sentinel-index idea has been credited in more than one cycle (cycle 1 v003, cycle 2 v001), so it is the most-revisited candidate, though its size benefit remains unmeasured.

## Recurring failures

- Per-op time and metadata size have never been scored on any lineage in cycles 1 to 3. Both goals behind the layout hypotheses are therefore untested, and Winner status has rested on hit-rate alone.
- Winner credit has been given without a reported gain over the parent (cycle 2 v001, cycle 3 v001 and v002). The selection criterion needs to be stated and applied consistently.
- Infrastructure failures (a session limit with no tokens recorded, and a missing insight file in cycle 1) removed strategies from evaluation before they were tested. Their approaches are unjudged rather than rejected.

## Open questions

- Does the sentinel index reduce per-entry metadata once measured, and does its check add per-op cost that offsets the saving?
- Does the relink skip or index pre-sizing reduce per-op time at all?
- Did the ghost-based admission filter ever engage on the trace? If not, which workload would exercise it, and does it improve hit-rate enough to justify its bookkeeping cost?
- Which parent is each reported hit-rate compared against? Lineage is ambiguous in cycle 2 and must be pinned down before comparisons.

## Per-policy notes

- LRU (all cycles so far): every variant tried has either preserved eviction order or been a behaviour-level idea (ghost-based admission) that has not yet shown a measured hit-rate change.
- LRU, eviction-order-preserving layout changes: hit-rate stays flat against the parent. Their value depends entirely on size and per-op measurements, which are still missing.
- LRU, admission-changing ideas: the only hit-rate-changing bet so far. Its result is uninterpretable until it is confirmed to have engaged on the trace.
