## Recurring wins

- Policy-neutral representation changes are the most reliable lever: narrowing arena links, packing flags into spare bits, and sentinel encoding shrink nodes without costing hit-rate (cycles 2, 3, 5).
- Deferring or bounding recency work mostly helps the put path (miss and eviction tails), not the get path (cycles 2, 4, 5).
- Keeping hashing and sketch work off the read path lowers read cost; the expense has been the hashing itself, not the admission decision (cycles 3, 4, 5).
- Admission gating and reuse-aware eviction have produced hit-rate gains on the scan-heavy workload, and those gains come from what is admitted or retained, not from list order (cycles 1, 3).

## Recurring failures

- Per-access overhead on shared paths keeps costing more than it returns: hashing on every access, and extra pointer work in the protected-side rescue that showed up in get as well as put (cycles 3, 4).
- Changes to arena access or index-conversion structure have not sped up get or put, and hoisted accessors made both slower (cycles 2, 3, 4).
- Per-node metadata additions without a clear hit-rate gain have not paid off (cycles 1, 5).
- Protection that rests only on a retained flag, without actually moving the head, recovers little of what list rotation provided (cycles 4, 5).
- Removing relinks on the hit path did not measurably improve get, so it is not a standalone goal (cycle 2).
- Per-operation and tail-latency gains from single runs are repeatedly unconfirmed, and several attributions are inferred rather than checked against the diff (cycles 1-5).

## Open questions

- Do the hit-rate gains from admission gating and bounded eviction hold on read-heavy workloads, other traces, and other capacities, or are they specific to the scan pattern?
- Can the protected-side protection be restored without list surgery on the shared path, using a different recency signal?
- Are the get and put movements in recent variants real, or layout and run-to-run noise that a re-measure would clear?
- Can per-access arena lookups be made cheaper without giving back the narrowed link width?

## Per-policy notes

- Segmented / protected-probationary LRU: the flag-only demotion retry cut per-operation cost and gave back a small share of hit-rate; the lost MRU rotation appears to be the protection itself. Credit-based rescue recovered only part of that protection.
- Second-chance / referenced-bit eviction: a bounded scan traded a small hit-rate loss for a more predictable miss-path cost. The deferred referenced-bit scheme lost some hit-rate on re-referenced keys; a single rescue did not close the gap.
- Admission filter (frequency sketch): the clearest hit-rate gain in the cohort, by rejecting one-hit keys at capacity. Its cost was hashing on every access. A shared cheap hash and write-only sketch updates lowered read cost, with a small hit-rate loss from dropping read-side popularity.
