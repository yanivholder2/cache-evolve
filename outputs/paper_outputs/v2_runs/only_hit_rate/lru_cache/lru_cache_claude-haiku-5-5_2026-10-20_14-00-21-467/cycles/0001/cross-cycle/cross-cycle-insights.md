## Recurring wins

- None yet confirmed across multiple cycles; only cycle 0001 is in the ledger.
- Single-cycle signal: a representation-only change (narrower index width via sentinel) kept hit-rate unchanged from its parent. Treat as promising, not established.

## Recurring failures

- Infrastructure failures, not strategy failures, have removed variants from evaluation so far: a session limit ended one run with no tokens recorded, and another produced no insight file. Track whether this recurs before drawing conclusions about the approaches involved.
- Winner credit has outrun measurement: a variant was named Winner on hit-rate alone while its stated goal (per-entry metadata size) went unscored. Require the stated goal's dimension to be measured before crediting a win.

## Open questions

- Does the index-width reduction shrink per-entry metadata once measured, and does the sentinel check add per-op cost that offsets the saving?
- Do the other hypotheses (H01–H03 broadly) preserve eviction order, or do they change it and need a separate hit-rate stability check?
- Which rejected variants (session limit, missing output) would produce evidence if re-run with their strategies unchanged?

## Per-policy notes

- LRU: all evidence so far is LRU. Representation-only changes that preserve eviction order have the only cohort-level signal; no policy-level conclusion is possible from one cycle.
- Other eviction policies: no data in the ledger yet.
