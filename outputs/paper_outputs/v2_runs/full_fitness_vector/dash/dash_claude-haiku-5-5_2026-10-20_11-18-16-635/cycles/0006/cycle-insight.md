# Cycle 0006 Insight

## Cohort outcome

The cohort tried three kinds of change: caching the eviction victim per bucket as an incremental minimum (v001), packing the `DashSegment` fields to shrink the segment (v002), and re-admitting recently evicted keys from a ghost ring (v003). Only the packed layout produced a measurable reduction, and it came with per-op regressions. The victim cache and the ghost ring both made the put path more expensive, and the ghost ring also lost hit rate. All three are labelled Winners, but none of them has a clean case on the dimensions the cohort measured.

## Patterns

- Maintaining the lowest-counter victim incrementally (v001) did not pay for itself. Hit rate and metadata size matched the full-scan parent, while put and get tails got worse. This answers the cycle 0005 question about a cheaper way to find the victim: keeping the minimum up to date on every counter write costs more than the scan it replaces, so the scan itself should stay.
- The layout change (v002) delivered the footprint reduction it targeted and left eviction order alone, which supports the cycle 0005 and 0004 view that layout and replacement are separable. But the packed fields added decode work on the get and promotion paths, and both operations slowed. A real size saving is not yet a usable win.
- Ghost-ring re-admission (v003) lost a small amount of hit rate on workload-a. The re-admitted keys appear to displace residents without adding re-referenced hits, which is the same displacement concern raised about promotion in cycle 0003. Its put-path bookkeeping on every insert and eviction is an added cost with no offsetting gain.
- Per-op movements again do not match their mechanisms. v003's reported get gain sits on a path the hypothesis did not touch, and the insight itself calls it unconfirmed. This is the same pattern flagged in cycles 0001 to 0005.
- Every hit-rate signal in this cycle comes from workload-a. The second-trace and stash-dominant checks requested in cycles 0003 to 0005 were not run here.

## Open questions

- The Winner basis for all three variants is still not visible. v001 and v002 record per-op regressions, and v003 records a hit-rate loss.
- Can the packed `DashSegment` keep its footprint reduction without the get and promotion decode cost, for example by packing only fields that are not read on the hot path?
- Does the ghost ring's cost show up in metadata size once its bytes are counted? It is currently unmeasured, so the put-path cost is not fully accounted for.
- Does full-scan lowest-counter eviction (cycle 0004 v002) keep its hit-rate gain on a second trace? That check is still outstanding.

## Continuation

Next cycle should drop incremental victim caching (v001) and keep the full scan as the victim-selection base, and it should test the full-scan frequency eviction on a second, non-workload-a trace as the cycle 0005 continuation asked. Pursue the packed layout (v002) only if the next variant removes the decode work from the get and promotion paths while keeping the smaller segment. Drop ghost-ring re-admission (v003) unless a later variant shows a hit-rate gain and reports the ring's bytes in metadata size.
