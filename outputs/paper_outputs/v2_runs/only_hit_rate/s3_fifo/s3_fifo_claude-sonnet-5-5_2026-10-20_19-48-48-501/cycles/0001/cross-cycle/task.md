# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants were single-point tweaks to baseline S3-FIFO admission and ghost policy: a looser small-to-main gate (v001), ghost tracking of main-queue victims (v002), and initial freq=1 on ghost hits (v003). Only v002 won, and its hit-rate gain was marginal. The other two were slightly worse than baseline.

## Patterns

- The two losers both made main more permissive (looser promotion, pre-protected ghost re-admits). Both added pollution that displaced useful residents. Baseline's strict filtering looks well tuned for this LIRS workload.
- The only winner added information (more ghost coverage) without changing admission or protection rules. It was safe but nearly neutral, since baseline main eviction rarely drops soon-to-be-reused items.
- Metadata-size and per-op-time were missing from every fitness vector. Cost claims for all variants are unverified.
- All three variants edited a single branch or condition, so none explored structural levers.

## Open questions

- Why were the metadata and timing dimensions not reported, and can they be restored so ghost-size costs can be judged?
- Would a stricter gate or a different small/main split help, given that looser gating hurt?
- Is the v002 gain stable across traces, or is it noise on LIRS?

## Continuation

Build on v002 (main-victim ghosting) and drop further loosening of promotion or ghost-hit freq, which both hurt. Next hypotheses should probe other levers: small/main size ratio, ghost capacity, stricter promotion, or main-eviction policy. They should also be combined with v002. Make sure metadata and per-op-time are measured.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
