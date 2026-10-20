# Cycle 0001 Insight

## Cohort outcome

The cohort tried layout-level changes to the LRU node representation, aiming to shrink per-entry metadata without altering eviction order. Only v003 (index-width reduction using a sentinel instead of `Option<usize>`) produced a usable result and was named a Winner, but its evidence is incomplete: the size and per-op-time effects are unmeasured, and the other two variants produced no usable evidence.

## Patterns

- The one evaluated variant held hit-rate unchanged from its parent, which is consistent with a change that should not alter eviction order. This is the only cohort-level signal so far, and it does not show any size benefit.
- The Winner's credit rests on a dimension it was not measured on. Its fitness reflects hit-rate only, so the metadata-size goal went unscored.
- Two variants were rejected for infrastructure reasons, not strategy: one hit a session limit and recorded no tokens, the other produced no insight file. Nothing can be concluded about their approaches.

## Open questions

- Does the index-width reduction actually shrink per-entry metadata once measured, and does the sentinel check add per-op cost that offsets it?
- Would the other hypotheses in this cycle (H01–H03 broadly) have produced comparable hit-rate stability, or do they touch eviction order and therefore need a separate check?

## Continuation

Next cycle should re-evaluate v003's size and per-op-time dimensions before building further on it, and treat its size win as unconfirmed. Keep pursuing representation-only changes that preserve eviction order, and re-run the variants that were rejected for infrastructure reasons rather than dropping their strategies.
