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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three probation-segment variants were tried: shrinking probation further (v001), a stricter two-hit promotion rule (v002), and a minimal fixed probation share built directly on the baseline geometry (v003). The two Winners were the small-probation variants (v001, v003). The stricter promotion rule lost hit rate.

## Patterns

- Two independent routes both ended at a small probation segment (about 4 slots per bucket) and both won. The trend toward smaller probation has not clearly turned, but the gain from v001 is marginal and looks to be flattening.
- v003 reports a large gain over plain LRU without the cycle 1-3 additions (ghost rings, counters, adaptive state). This suggests the simple split plus small probation captures most of the benefit, and the extra bookkeeping from earlier cycles may be unnecessary.
- Delaying promotion hurt (v002). With a small probation segment, entries cannot wait for a second hit without being evicted. Together with cycle 0003's ghost result, first-hit promotion looks hard to beat here.
- Per-op-time and metadata-size were unmeasured for the fourth cycle running. Every claimed win is hit-rate only, and the cost of segment reordering and demotion on `get` is still unpriced.
- v003's comparison is against the baseline, not the current parent, so its size is not an increment over v001.

## Open questions

- Where is the actual minimum of probation size? Is going below 4 slots (for example 2, or 1) still better, or does scan resistance break?
- Does v003's simple split match the full accumulated stack, and can the older extra state be removed?
- Do the hit-rate gains survive once per-op-time and metadata-size are measured?
- Would a larger probation segment make a two-hit rule viable, or is first-hit promotion simply better?

## Continuation

Probe probation sizes just below the current point (and one above, as a check) to find the turn, and consider simplifying by stripping unneeded earlier state, using v003's lean split as a reference. Drop two-hit or other delayed-promotion rules in a small probation segment. Fix the evaluation so per-op-time and metadata-size appear in the fitness vector before adding any new bookkeeping.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three probes of the small-probation segmented LRU were tried: a fixed 2-slot probation (v001), a recency-stamp compare that lets stale protected entries be evicted (v002), and a lean positional 4-slot split rebuilt on the baseline with no extra state (v003). Only v003 was a Winner; v001 and v002 each lost a little hit rate against their parents.

## Patterns

- The probation-size trend has turned. Going below the slots/16 sizing slightly hurt, so the current size is at or near the optimum and further shrinking is exhausted.
- Eviction-side tweaks did not help. Letting raw recency override probation-first eviction cost hit rate, because the segment boundary already encodes reuse evidence that one last-access stamp does not capture.
- Together with earlier cycles (ghost history, delayed promotion, frequency windows, size tuning), most simple levers on this split have now been tried and found flat or negative. The simple structural split holds almost all the gain.
- v003 again shows that a lean positional split with no ghost, counter or adaptive state improves on plain LRU. Its comparison is against the baseline, not an increment over the current best.
- Per-op-time and metadata-size were unmeasured for the fifth cycle running. v003's "lean" claim (same footprint and speed as baseline) is still untested, and promotion/demotion reordering on hits is still unpriced.

## Open questions

- Does the lean split really keep baseline metadata size and per-op-time, and do the gains survive once those are measured?
- Is there any remaining hit-rate headroom in promotion or protected-segment handling, or is this design at its ceiling for the trace?
- Does the result hold on traces other than the LIRS-style one, or is the tuning specific to it?

## Continuation

Treat the lean 4-slot positional split (v003, with slots/16 probation sizing as the reference) as the base. Drop further probation-size shrinking and recency- or stamp-based victim overrides. The next cycle should first get per-op-time and metadata-size into the fitness vector; after that, try only mechanisms that add no per-entry state, or check robustness on other workloads.


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
