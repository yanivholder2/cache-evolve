# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants started from baseline and each changed one thing: the partition split (v001), the graduation rule (v002), and the data structures with policy left unchanged (v003). The Winner was v001, which shrank the small queue to the paper's 10%. v002 lowered the graduation threshold and lost a little hit-rate. v003 kept hit-rate exactly where it was.

## Patterns

- On workload-a, giving more room to the main queue and the ghost queue (which grows with main) helped. Making it easier to enter main did not. The policy rewards keys that have shown repeated reuse, not keys that were reused once.
- The ghost queue already acts as a safety net. A key that misses graduation can still get back into main later, so both a shorter probation window (v001) and an earlier promotion (v002) change less than expected.
- Each variant reported hit-rate only. Neither per-op-time nor metadata-size was measured anywhere in the cohort, so the stated purpose of v003 and the ghost-scan cost risk in v001 are both still untested.
- v002 changed two things together (the threshold and a frequency reset on graduation), so we can't tell which one caused the regression.

## Open questions

- Does the larger ghost queue in v001 hurt per-op-time while the linear scans are still in place?
- Does v003's indexed slab actually lower get/put latency, and what does it cost in metadata-size?
- Was v002's regression caused by the lower threshold, by the frequency reset, or by both together?

## Continuation

Use v001's 10% split as the new policy baseline, and port it onto v003's behaviour-neutral indexed structure so that hit-rate gains and per-op-time gains can stack. Every variant must report all three dimensions. Don't pursue earlier graduation as it was tried here. If you revisit it, test the threshold change alone, with no frequency reset. Partition sweeps around 10% and ghost-queue sizing that is decoupled from main are worth probing next.


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
