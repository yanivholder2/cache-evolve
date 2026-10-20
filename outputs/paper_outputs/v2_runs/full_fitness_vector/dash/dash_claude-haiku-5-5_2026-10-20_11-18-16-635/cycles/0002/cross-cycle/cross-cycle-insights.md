## Recurring wins

- Protecting hot residents at eviction time, not at insert time, is the only direction that has reported a hit-rate gain (cycle 0002, victim choice biased by a small saturating access counter). This fits the cycle 0001 concern that promotions can displace hot target residents.
- Get-path reordering that checks target and probing tiers before the stash has a clear latency mechanism (cycle 0001). It is not yet confirmed on stash hits or across reruns, so treat it as a candidate, not a settled win.

## Recurring failures

- Insert-position changes have lost hit rate in both cycles. MRU promotion protects the promoted key but can displace hot target residents (cycle 0001, v001 hit-rate loss). Gating insert position on bucket fullness also cost hit rate (cycle 0002, v001), so promotions into full buckets still need the back-of-queue protection.
- Stash-path bookkeeping does not pay on these workloads. An occupancy fast path with write-side maintenance raised get and put latency (cycle 0002, v002), which suggests stash buckets are rarely empty.
- Per-op and put-side latency claims keep moving in ways the changed code paths do not explain (cycle 0001 and cycle 0002). Latency gains should not be credited until re-measured.
- Winner labels have been given despite recorded hit-rate or latency regressions (cycle 0001 v001, cycle 0002 v001 and v002). The criterion appears to weight latency over hit rate, and it is not stated in the insights.

## Open questions

- Is the v001 hit-rate loss from cycle 0001 stable across traces, and does it appear only when target buckets are full?
- Does the victim-selection gain from cycle 0002 v003 hold on workloads other than workload-a?
- Does the get-path reordering's latency gain survive stash hits, which pay for the two earlier misses, and reruns?
- Are the unexplained put-side and per-op movements measurement noise?
- What criterion makes a Variant a Winner when it regresses hit rate or latency, and is that trade-off intended?

## Per-policy notes

- Insert position (MRU promotion, fullness-gated front insert): hit rate has fallen in both cycles. Do not pursue further insert changes without a guard against displacing hot target residents.
- Victim selection (saturating access counter, frequency-aware choice): the only hit-rate gain so far. Next step is to check it on more than one workload before building on it.
- Get-path lookup order (stash scan versus target and probing scans): the mechanism is latency, not hit rate. Confirm on stash hits before counting it.
- Stash occupancy handling: no gain observed; the fast path added latency on both get and put.
