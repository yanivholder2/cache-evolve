## Recurring wins

- Making one isolated change per variant on the current parent has given a clear Winner or a clean negative result in every cycle (cycles 1, 3, 4, 5). Variants that bundled several changes (cycle 2 v001) could not be traced back to a single cause.
- The lineage so far is: 3/4 normal-stash topology (c1), then a frequency-aware 8-slot LRU-tail victim with halve-on-cap ageing (c2), then ghost-credited admission (c4), then a direct-mapped, capacity-sized ghost (c5). Each step stacked on the previous one with no regression.
- Changes that act inside a bucket (victim choice, admission credit) move the small cache size. Topology alone moved only the large size.
- Admission-side memory (the ghost) is the only lever that has reached hm_0's long-gap reuse. Ghost reach (capacity) mattered more than bonus strength.
- Making the ghost lookup O(1) improved hit-rate and also argues for lower cost. Structural simplifications that also add reach are worth preferring.

## Recurring failures

- MRU or Back landing on promotion: it hurt web_0 in c2 (bundled with other changes) and again in c3 (applied alone). Front landing is part of how the tail-scan filter works, because it keeps arrivals inside the window.
- Insert-position changes once eviction reads the counter: gated demotion (c3) was redundant with the scan.
- Tuning the tail-scan window: narrower (c3) loses the frequency filter, wider (c4) turns the policy into coarse LFU. Window 8 is the peak.
- Counter ageing: faster ageing (c3 cap 7, c4 decrement-per-eviction) drains protection, and removing the stash-hit halving (c5) makes items stick in target. Counters must keep ageing at roughly the current rate.
- Normal/stash ratio sweeps past 3/4: shrinking the stash starves promotion on hm_0. The ratio has peaked.
- Narrowing the ghost fingerprint: false matches cost web_0.
- Measuring cost: per-op-time and metadata-size have been requested every cycle and reported in none (5 cycles). Every cost claim is still argued from code.

## Open questions

- Does a +2 ghost-hit bonus add on top of the capacity-sized ghost, now that the ghost fires more often, if fingerprints stay full width?
- Does a ghost larger than capacity (e.g. 2×), or a different admission rule, reach the large-size hm_0 reuse, which every ghost change so far has left untouched?
- Can stash ageing be moved rather than removed (age on insertion, or reset/halve a demoted item's counter on entry) to protect ghost credits without stickiness?
- Would window width or ageing rate scaled per bucket type or by cache size capture the small-hm_0 frequency preference without losses elsewhere?
- Is there any lever for web_0? Nothing tried in five cycles has produced a gain there.
- What are the actual per-op-time and metadata-size of the scan, the halving, and the ghost table?

## Per-policy notes

- Topology (normal/stash split): 3/4 normal is the best split. Stash entry is the only path into the protected tier, so stash size bounds promotion. The split is spent as a lever.
- Tail-scan victim (frequency filter): 8-slot window plus slow halve-on-cap ageing is near optimal. Its gains are concentrated in small-size hm_0, and it depends on Front landing.
- Promotion/demotion placement: Front landing is required. MRU variants and counter-gated placement have been neutral or harmful since the filter was added.
- Ghost admission: reach matters more than bonus. Capacity-sized direct-mapped is the current best. Gains appear only at small size on hm_0. Fingerprint width is a real cost on web_0.
- Traces: hm_0 is the most responsive trace in both directions. web_0 is nearly inert and reacts mainly to harm: MRU landing and fingerprint false matches.
