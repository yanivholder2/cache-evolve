## Recurring wins

- Eviction-neutral changes (sentinel indices, lookup collapse, hasher swap, pre-sized index, head-hit relink skip) have repeatedly held hit-rate equal to their parent on workload-a at size 2048 (cycles 0001–0004). They are safe, but no cycle has yet shown a measured benefit from them.
- A frequency signal used only as a bounded tie-breaker among recent tail entries gave the one slight hit-rate gain in the ledger (cycle 0007). It has not yet been reproduced on another trace or size.

## Recurring failures

- Changes that alter recency order lowered hit-rate on workload-a at size 2048: segmented probation/protected lists and second-chance referenced bits (cycle 0005), and second-hit promotion (cycle 0006). Strict recency ordering appears to matter on this trace.
- Tail insertion of new keys has no consistent sign: a gain (cycle 0005), losses (cycle 0006), and an exact null (cycle 0007). Until its engagement is verified, no cycle can be cited as settling it.
- Admission gating (ghost-set filter, count-min gate) and hit-counter changes (decay, saturating caps) produced flat or slightly lower hit-rate (cycles 0007, 0008). None of these insights confirms the gate or counter ever engaged.
- Winners were named without evidence on the dimension they were chosen for. Hit-rate fell or stayed flat for several Winners (cycles 0001–0003, 0005, 0006, 0008), while per-op time and metadata size were never recorded.
- Several variants were lost to infrastructure (session limit, missing insight file in cycles 0001 and 0004). Their strategies were never tested.

## Open questions

- Per-op time and metadata size have been missing from every cycle. Until they are recorded, no size or speed claim (sentinel indices, saturating counters, hasher swap, hit-path collapse) can be confirmed.
- What is the selection criterion for Winners? Several were chosen without a measured gain on any dimension, so the lineage they seed is in doubt.
- Does tail insertion help or hurt against a fixed parent? The cycles 0005–0007 results cannot be reconciled from the insights.
- Does the bounded frequency tie-breaker (cycle 0007) hold on a second trace or cache size? Does any counter-based or admission idea help on traces other than workload-a?
- Did the ghost filter and count-min gate ever reject a candidate? If not, admission ideas are untested rather than refuted.
- The H03 label has referred to different mechanisms across cycles. It needs one definition before lineages cite it.

## Per-policy notes

- LRU (strict recency, move-to-front): the baseline is hard to beat on workload-a. Layout and index changes preserve it; the only recency-order change that did not hurt is tail insertion, and its evidence conflicts.
- Segmented LRU (probation/protected): lowered hit-rate as written (cycle 0005). Only the protected cap or demotion rule might still carry headroom, and it has not been tested in isolation.
- Second-chance / CLOCK-style referenced bits: lowered hit-rate as an approximation of LRU (cycle 0005). Dropped as written.
- Frequency-aware eviction (least-hit within a bounded tail window): the only slight hit-rate gain seen (cycle 0007). Its per-entry counter cost is unmeasured. Keep as the lead candidate, pending a second trace and cost data.
- Counter-capped or decaying frequency (saturating width, decay of victim-scan counters): flat or slightly down (cycle 0008). Its footprint benefit is unmeasured, so it has no positive evidence yet.
- Admission filters (ghost set, count-min gate on the LRU tail): flat in both tests (cycles 0007, 0008). Engagement is unverified, so treat as untested.
