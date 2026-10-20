## Recurring wins

- Behaviour-preserving layout edits (sentinel index in place of optional links) are the only changes that have produced usable results in more than one cycle. Hit-rate stayed flat against the parent each time, which fits a change that does not alter eviction order. The size benefit they were written for has never been measured, so treat this as "safe", not "proven".

## Recurring failures

- Fitness is scored on hit-rate only, so goals that matter for these changes (metadata size, per-op time) go unscored. Credit is then given on a dimension the change was not built for.
- Per-op time has not been measured on any lineage across any cycle.
- Several variants have been lost to infrastructure problems (session limits, missing insight output), not to their strategy. Their approaches have never been properly tested.
- Behaviour-changing ideas have not yet produced usable evidence, so no cohort has shown whether they pay off.

## Open questions

- Does the sentinel-index change actually shrink per-entry metadata, and does its sentinel check add per-op cost that offsets the saving?
- Which parent is the hit-rate "gain" measured against? The cycle 2 records disagree, and no comparison is valid until this is pinned down.
- Is the sentinel change redundant across cycles? If the current winner already contains it, the next layout step must measure something new rather than re-apply it.
- Does ghost-based admission improve hit-rate enough to cover its bookkeeping cost on per-op time?
- Do the rejected variants from cycle 1 and the pre-sized index allocation idea from cycle 2 still hold up when re-run on a clean lineage?

## Per-policy notes

- LRU (the only policy in the ledger):
  - Layout-only changes that preserve eviction order are the only ones with usable evidence so far. Their hit-rate effect is neutral; their size and per-op effects are unknown.
  - Admission-filter ideas (ghost-based) change what stays resident and would need hit-rate and per-op judgement together. None has been evaluated yet.
  - No cross-policy comparison is possible from the ledger.
