# Variant v003 Insight

## Helped

- hit-rate: Both hm_0 cells, small and large, rose by the same small margin, and web_0-small moved up at noise level. No cell lost more than the hypothesis's 0.0005 tolerance, so the "small-cache loss" refutation (credited keys crowding out fresh arrivals) did not happen. On hm_0, a meaningful share of stash victims that were never hit come back while their fingerprint is still in the 16×4 ghost set. The one-shot `lfu_counter = 1` credit then lets the next stash hit pass the two-hit gate one touch sooner. Because the credit is single-use and stops at the gate, it does not dilute the cache.

## Hurt

- hit-rate: web_0-large, the cell this hypothesis targeted, did not rise. It moved slightly down, within noise, so the claim fails the confirmation bar outright. In the hypothesis's own terms that is "refuted, no gain". The miss on web_0-large is not that keys came back and were judged brand new. Either they come back after more than one ghost lifetime, since a 64-entry per-segment ring gets overwritten quickly at 128K capacity, or they do not come back at all. In both cases the trace is capacity-bound, not admission-bound.
- per-op-time: All four latencies rose, including get, whose path the change does not touch. That points to a slower host in this session, not to the code, and it matches the parent's own caveat that its timings were host-dependent. Put rose clearly more than get, though. Put p99 used to sit well below get p99 and is now about equal to it. That relative shift fits the real new work on the put/miss side: a 4-way u16 ghost scan on every stash insert, plus a ghost write on each never-hit victim. A strict check against the "put p99 near same-session H01" criterion needs H01's numbers, which this comparison does not have.

## Takeaway

A one-shot ghost credit for never-hit stash victims is cheap in hit-rate risk: no cell lost, and Item and Dash both stay at 24 bytes. But it only pays where returns happen within a short window (hm_0). It does nothing for web_0-large, which closes stash-gate admission as the lever for that cell, as the hypothesis said a flat result would. Before keeping it for hm_0's small gain, check whether the extra put cost holds up against a same-session control. Caveat: I could not list or open the overlay source in this session, so the code-level reasoning assumes H03 was implemented as written.
