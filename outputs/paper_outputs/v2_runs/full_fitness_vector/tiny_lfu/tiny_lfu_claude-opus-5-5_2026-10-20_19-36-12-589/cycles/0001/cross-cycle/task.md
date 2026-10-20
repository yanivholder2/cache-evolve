# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants branched from the baseline, each along a different axis: resident-list structure (W-TinyLFU window + SLRU, v001), sketch freshness via periodic aging (v002), and hashing/sketch layout (v003). The winner, v003, was a pure efficiency change. It lowered get and put latency at both p50 and p99 and shrank the cache struct slightly, while leaving hit rate unchanged.

## Patterns

- Hit rate did not move in this cycle. Both completed variants kept the baseline `CountMinSketch` counting rules, and both stayed within noise on hit rate. That suggests the admission signal, not the list structure, is what caps hit rate right now.
- Per-op cost in the baseline came mostly from hashing, not from cache logic. Removing the repeated per-row SipHash calls brought large latency gains, and the put tail gained most because it runs the admission contest.
- Adding structure has a cost that only pays off if hit rate rises with it. v001's extra region tags, per-list arrays and per-hit relinking made memory and latency worse and gave almost no hit-rate gain.
- v002, the aging hypothesis, never ran. It was rejected because of an infrastructure failure (an HTTP 429 from Maven while downloading the Caffeine simulator), not because of anything in the hypothesis. Its idea is still untested.

## Open questions

- Does sketch aging/reset (with a lower saturation cap) actually raise hit rate on workload-a, and does it make a window/SLRU structure worthwhile once the frequency signal stays fresh?
- Is the unkeyed fast hasher an acceptable risk if a trace has adversarial or collision-prone keys?
- Would a larger window than about 1% change v001's outcome, or does the window size matter little while the sketch never ages?

## Continuation

Build on v003, since it is now the cheap-hashing baseline. Re-submit the H02 sketch-aging idea on top of it, because the infra failure means it was never evaluated, and target hit rate through the frequency signal. Drop further resident-list restructuring such as SLRU or window variants until aging has been shown to move hit rate. If a structure is revisited later, it must keep the per-hit relink cost low.


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
