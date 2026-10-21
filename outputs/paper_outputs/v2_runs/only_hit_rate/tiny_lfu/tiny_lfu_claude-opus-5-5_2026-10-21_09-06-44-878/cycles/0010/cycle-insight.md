# Cycle 0010 Insight

## Cohort outcome

All three variants came from the same parent and tested frequency-only levers that had not been tried before. v001 moved a victim that survived the contest to probation MRU, rotating the window CLOCK-style. v002 added the protected LRU head as one extra eviction candidate. v003 gave cache hits double weight in the sketch. v001 is the Winner, but only on a gain within noise. v002 was flat and v003 lost a little. This is the sixth cycle in a row with no real gain.

## Patterns

- Changing which entries are compared does nothing while every entry is scored on unaged lifetime counts. Rotating the window and adding a protected candidate both widen or reshuffle the set of victims. The minimum estimate in that set stays put, because old probation entries share similar near-floor counts and protected keys can never fall below that floor. Victim-window mechanics are now fully tested: depth (0003, 0009), rotation, and protected reach.
- v003 checked the opposite direction, sharpening the incumbent signal instead of adding recency. It still lost. Weighting hits ties a key's estimate to whether it happened to be resident, so a feedback loop starves keys that were rejected. Both directions have now been tested: tilting the contest toward newcomers or recency loses (0007–0008), and tilting it toward incumbents loses too. The plain access count is already the right currency at the right balance.
- Cycle 0007 asked whether "stale-high" protected keys could be reached. v002 settles it. They are stale in recency but not in count, so a contest on frequency alone can never pick them. That leaves recency signals as the only route, and cycle 0008 refuted those on this design.
- The v001 and v002 takeaways both say aging or decay is the next step. Cycle 0008 tested exactly that on the current design and lost. Treat these suggestions as already refuted, not as open leads.

## Open questions

- Is metadata-size or per-op-time scored yet? If it is, packing counters to 5 bits (cap 31, inert on hit-rate per 0009) is still the only actionable lead. v001's rotation is free on metadata, but whether it costs per-op-time has not been checked.
- Are there other workloads where this family is not saturated?

## Continuation

Workload-a's hit-rate is at its ceiling for TinyLFU+SLRU. The admission contest, victim selection (depth, rotation, protected reach), segment movement, sketch range, hit weighting and aging have all been tested, and none gave a real gain. Do not propose more workload-a hit-rate tweaks, including the aging that the variant takeaways suggest. Spend the next cycles on cost dimensions (5-bit counters, per-op-time) or on other workloads, and stop if neither is scored.
