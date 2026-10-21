## Recurring wins

- Mechanisms that add scan resistance beat strict LRU on hit-rate: segmented LRU (cycle 1), one-bit CLOCK second-chance (cycle 2), and a larger protected share (cycle 3).
- Tuning the segmented design's protected share has a consistent direction across two cycles: smaller loses, larger gains.
- Gains are small but repeatable in sign, and the invariant gate accepts departures from strict LRU.

## Recurring failures

- Weakening the protected segment (shrinking it, or sending ghost hits straight to protected) lowers hit-rate.
- The gentler ghost list (probation-MRU re-admission) removes the regression but adds nothing. It likely rarely changes a decision on the current trace.
- Strict-LRU-preserving micro-optimizations (compact layout, hot-path rewrites) cannot win on hit-rate. They are neutral there, and their intended benefit is unmeasured.
- Adding frequency bits to CLOCK is incremental, not transformative.
- Metadata-size and per-op-time have been missing from every fitness vector for three cycles. Size and speed claims (CLOCK, compact nodes, ghost-list cost) are all unverified, so selection has been on hit-rate alone.

## Open questions

- Is the metadata-size and per-op-time gap a harness or configuration fault? Resolving it comes before any further variants.
- Do the gains hold on other traces and cache sizes? Everything so far rests on one scan-heavy trace at one size.
- Does the protected-share gain plateau or reverse beyond 90%? Probation must still absorb scans.
- Can the protected-share tuning or compact/fast-path ideas combine with CLOCK or frequency mechanisms?
- Would a ghost list help on traces with long-distance reuse?

## Per-policy notes

- Segmented LRU: a large, stable protected set is favored. Ghost variants have not paid off. Per-node flags, a second list and demotion work have unmeasured costs.
- CLOCK (one-bit): wins a small hit-rate gain with a plausible size and speed advantage that is still unverified.
- Frequency CLOCK: modest gain, no win. It is similar to one-bit CLOCK.
- Strict LRU: the baseline. Further micro-optimizations are not worth cycles until size and time are measured.
- Single-constant tweaks should wait for a second validating trace.
