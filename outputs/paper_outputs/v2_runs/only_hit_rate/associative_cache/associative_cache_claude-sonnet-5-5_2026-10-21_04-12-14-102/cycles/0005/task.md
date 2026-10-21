# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0005

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0005/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
