## Recurring wins

- Protecting hot residents at eviction, not at insert, is the most consistent hit-rate direction. A frequency-aware victim choice (saturating access counter) was the only hit-rate gain in cycle 0002 and again in cycle 0004.
- Deferring stash-to-target promotion has held a hit-rate gain across two cycles (cycle 0003, cycle 0004). Its cost sits in first-hit state on stash hits, and cheaper bookkeeping keeps the gain while recovering the get tail.
- Keeping single-touch entries out of target buckets, in the form of a deferred promotion, is the only admission-side change with a small positive signal. Blocking first-touch admission outright lost hit rate, so admission should not be gated broadly.

## Recurring failures

- Changing promotion insert position based on bucket fullness or MRU placement has lost hit rate (cycle 0001, cycle 0002). Promotions into full buckets still need the back-of-queue protection they already get.
- Blocking first-touch admission lost hit rate (cycle 0003). Early admission matters on the measured trace.
- Stash occupancy checks on the get path, maintained on writes, raised get and put latency (cycle 0002). Stash buckets appear rarely empty, so fast paths here cost more than they save.
- Recency tie-breaking among equal eviction counters moved no hit rate and added eviction-path cost (cycle 0003).
- Record repacking is hit-rate neutral (cycle 0004). It does not change eviction outcomes.
- Per-op latency claims have not held up across any cycle (cycles 0001 to 0004). Put-side and per-op movements repeatedly exceed what the edited code path can explain. Treat latency gains as unconfirmed until re-measured.

## Open questions

- Does the frequency-aware victim gain hold on a second, non-workload-a trace? Does the put-side cost of the lowest-counter search on full buckets grow with bucket size?
- Does the deferred stash-to-target promotion keep its hit-rate gain on a stash-dominant trace?
- Does any admission gating help on scan-heavy traces? Every signal so far comes from workload-a, and the trace types earlier cycles asked for are still unmeasured.
- Is the hit-rate loss from MRU or fullness-gated insert stable across traces, or does it come only from full buckets?
- What explains Winner labels on variants with recorded per-op regressions and no hit-rate gain (cycle 0002, cycle 0003)? Winner criteria appear to weight latency over hit rate, and it is unclear whether that is intended.
- Are the unexplained put-side and per-op movements measurement noise? Re-measuring with single changes applied would settle it.

## Per-policy notes

- Insert and promotion (MRU insert, fullness gating, first-touch admission gating, deferred stash-to-target promotion): MRU or fullness-based insert placement loses hit rate. Broad admission gating also loses hit rate. Only the deferred promotion has shown a gain, and that gain was measured on one workload.
- Victim selection (saturating access counter, frequency-aware choice): the most reliable hit-rate direction in the ledger, though only on workload-a so far. Recency tie-breaking among equal counters is neutral for hit rate and adds cost.
- Stash path (occupancy fast paths, get-path tier ordering): occupancy checks have added latency. Get-path reordering was the one latency-motivated change, and its gain is unconfirmed.
- Record layout (repacking): hit-rate neutral, so layout and replacement policy look separable. Any per-op gain from layout is unexplained.
