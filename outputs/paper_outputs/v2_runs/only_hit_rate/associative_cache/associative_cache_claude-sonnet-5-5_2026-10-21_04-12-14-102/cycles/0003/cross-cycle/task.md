# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three variants tried: wider buckets (geometry, v001), cheaper one-step hit promotion (v002), and 2Q-style half-promotion for scan resistance (v003). Only the geometry change (v001) won, with a small hit-rate gain. Both policy-tweak variants slightly lowered hit rate versus plain move-to-back LRU.

## Patterns

- Only hit-rate was reliably measured. Per-op-time and metadata-size were missing from every variant's fitness vector, so the speed and size claims in all three hypotheses went untested.
- Both variants that weakened promotion in short buckets (v002, v003) lost hit rate. Full move-to-back recency seems hard to beat at small associativity on this LIRS trace.
- The geometry gain was marginal, so hash imbalance across sets is a minor source of misses. Most misses look like capacity or policy misses.
- Winning by a small hit-rate margin is unproven, because the likely timing cost of 64-way scans and shifts was never checked.

## Open questions

- Why was per-op-time not reported, and can the evaluation be fixed so speed-targeting hypotheses can be judged?
- Does the wider-bucket gain survive once get/remove/push cost is measured?
- Would scan resistance work with a separate probationary region instead of a shallower promotion within one short bucket?

## Continuation

Build on v001's geometry, but first make sure per-op-time and metadata-size are measured so trade-offs can be judged. Drop promotion-weakening ideas (transposition, midpoint) in plain LRU buckets. Next, try policies that change eviction or admission (e.g. a separate probationary segment, frequency-aware victim choice) rather than hit-path promotion.


### Cycle 0002

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three refinements of the probation/protected split were tried: a smaller probation segment (v001), a per-bucket ghost ring that promotes ghost hits straight to protected (v002), and an adaptive 2Q-style probation target with ghost history (v003). Only the smaller probation segment won, with a small hit-rate gain. The ghost-promotion variant did not improve on its parent.

## Patterns

- Tuning the split size gave only a small gain, so most of the benefit comes from having the split at all. This matches cycle 0002.
- Ghost-history ideas did not add value on top of the split. A ghost hit looks like weak reuse evidence on this trace and promoting on it can displace entries that had earned protection.
- v003's insight compares against plain LRU and credits the whole split, not an increment over the current parent. It is hard to read as a new gain, and it did not win.
- Per-op-time and metadata-size were unmeasured for the third cycle running. Every added piece of state (ghost rings, adaptive targets, markers) is still unpriced.

## Open questions

- Where does the smaller-probation trend turn? Is a smaller share still better, or is the optimum near the current one?
- Does a fixed probation size match an adaptive one, making the extra ghost state unnecessary?
- Would a stricter promotion rule (ghost hit plus a second signal) rescue ghost history?
- Do the gains survive once speed and metadata cost are measured?

## Continuation

Build on v001's smaller probation size and probe a few more sizes to find the turning point. Drop ghost-hit direct promotion and further ghost-ring variants unless promotion is made stricter. Fixing the evaluation so per-op-time and metadata-size appear in the fitness vector should come before adding any more bookkeeping.


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
