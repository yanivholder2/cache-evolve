# Cycle 0002 Insight

## Cohort outcome

Three eviction/admission ideas were tried on the wide-bucket geometry: a probationary segment (v001), frequency-aware victim choice (v002), and two-choice bucket placement (v003). Only the probationary segment won, with a clearly larger hit-rate gain than cycle 0001's geometry change. Frequency-aware victim choice left hit rate unchanged. Two-choice placement was rejected at the invariant gate.

## Patterns

- Admission control (separating one-touch entries from re-referenced ones) helped, while a small frequency window on the LRU end did nothing measurable. The scan-pollution benefit seems to need a structural separation, not a tie-break among a few old entries.
- Placement changes that break the "key lives in its hashed bucket" rule fail the slot-legality invariant, even on the baseline. Any multi-bucket placement scheme conflicts with the gate as written.
- Per-op-time and metadata-size were again unmeasured, so none of the added bookkeeping costs (partition counters, per-entry counters, longer scans) has been weighed against the hit-rate gains. This gap is now two cycles old.
- Policy, not geometry or hash balance, looks like the main limit on hit rate for this trace.

## Open questions

- Is the probation size (about a quarter of the bucket) near the best value, or does a smaller or larger share do better?
- Would a wider frequency window, or an actual probation-to-protected frequency test, add anything on top of the probationary segment?
- Does the probation win hold once per-op-time and metadata-size are measured?

## Continuation

Build on v001's probationary segment: tune probation size and try variants that combine it with frequency or ghost-history information for promotion. Drop small-window frequency victim selection and any placement scheme that moves keys outside their hashed bucket, unless the invariant is deliberately relaxed. Get per-op-time and metadata-size into the fitness vector before judging further trade-offs.
