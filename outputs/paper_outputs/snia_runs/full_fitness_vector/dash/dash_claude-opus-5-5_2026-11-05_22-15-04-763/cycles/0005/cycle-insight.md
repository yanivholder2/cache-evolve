# Cycle 0005 Insight

## Cohort outcome

All three variants built on cycle 0004 v002 (gate + MRU demotion + probing overflow + probing room gate), and each one tested a separate thread. v001 reversed the in-bucket scan (cost only). v002 allowed exactly one deferred promotion when target and probing are both full. v003 sent unreused demotions to the stash LRU end. There are no Winners. v002 was the only one to fix what it targeted: it brought back `put` p99, but the cost moved to `get` tail and to small-cache hit rate.

## Patterns

- Two levers are now ruled out. Scan direction does not affect `get` p50 (v001 kept hit rate identical, so this was a clean test), because the buckets that miss still need full scans. A liveness signal taken from a single normal-tier stay does not work either (v003 lost hit rate everywhere, most on hm_0 at the large size). MRU demotion for every demoted item stays.
- Deferral moves the cascade but does not remove it. The single deferral took the both-full cascade off `put` and put it on a later `get`. This matches cycle 0004: skipping a cascade just moves its cost somewhere else. The difference is that a limited deferral keeps stash draining, so `put` recovered rather than getting worse.
- Cache size decides whether stash residency helps. When an item stays in stash longer (deferral), the large cache gains because proven residents stay in place. The small cache loses because deferred items take up stash slots. hm_0 at the large size again rewards anything that keeps long-gap reusers.
- Every variant made at least one p99 slightly worse, including v001, whose change cannot add work to `put`. Small tail moves of this size may be partly noise.

## Open questions

- Which part of the common `get` hit path holds the p50 gap? The two remaining suspects are how many buckets each hit probes (stash checked first on every lookup) and the shift cost when an item moves to MRU.
- Can the deferral depend on stash pressure (defer only when stash has headroom) so the small-cache hit-rate loss goes away while the `put` p99 gain stays?
- Can the forced promotion on the third hit be made cheaper, or moved off the `get` path?

## Continuation

Take cycle 0005 v002 (single deferral) as the base if the next hypotheses target its `get` tail or its small-cache loss, for example a deferral gated on stash pressure. Otherwise stay on cycle 0004 v002. Aim `get` p50 work at how many buckets each hit probes and at the cost of moving an item to MRU. Drop scan-direction changes and any demotion placement that relies on reuse within a single normal-tier stay.
