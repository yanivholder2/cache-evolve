# Cycle 0009 Insight

## Cohort outcome

Variants tried put-path-only adaptive insertion depth (v001), a position-gated two-slot hit promotion (v002), and a fixed len/4 swap insertion with first-empty/in-place victim overwrite as a control (v003). v001 and v002 were Winners, but none shows a clear gain: v001 lost hit rate and time, v002 gained marginal hit rate at a latency cost, and v003 (not a Winner) got cheaper puts but lost hit rate.

## Patterns

- The Winner-versus-evidence mismatch continues, a seventh straight cycle. v001 reports losses on hit rate and on both latencies, and v002's own insight calls its trade weak.
- Moving the adaptive signal off the get path did not make it free. The put-side counter and data-dependent branch cost put time, and the victim-position proxy was a noisier signal than static len/4.
- The fixed-position control answers last cycle's question. Swap insertion at a fixed len/4 cuts put cost but loses hit rate, because the swap pushes the displaced mid-bucket entry to the tail where it is evicted next. Any earlier adaptive gain therefore came from the adaptive part, and v001's weaker proxy did not reproduce it.
- Position as an implicit "hit before" signal is free in metadata but not in time. Faster promotion for already-promoted entries again gives only marginal hit rate, matching cycles 0007 and 0008: transpose-by-one is hard to beat.
- Get latency again has no real gain; every extra compare or swap on the hit path made it slower.
- No variant grew metadata.

## Open questions

- What signal made the cycle 0008 adaptive scheme work (global hit/miss counter), and can it be updated cheaply without a per-get cost?
- Can swap insertion avoid sacrificing the displaced entry, e.g. by shifting rather than swapping into the tail, while keeping the put saving?
- Are any Winner statuses beyond noise, and do results hold on a second workload?

## Continuation

Drop victim-position proxies for adaptivity, position-gated promotion jumps, and fixed-position swap insertion as a hit-rate idea (keep it only as a put-cost option). Next, retry adaptivity with the cycle 0008 global hit/miss signal, or test a swap variant that does not demote the len/4 occupant to the tail. Add a noise check and a second workload, and keep discounting Winners whose own insight shows a loss.
