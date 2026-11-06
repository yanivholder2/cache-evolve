# Variant v003 Insight

## Helped

(none — metadata-size did not move, because the reuse bit is stored in the existing `lfu_counter` field.)

## Hurt

- hit-rate: Every trace and size went down. hm_0 at the large size went down most, and that was the hypothesis's own test case, so the claim fails. "Unreused" is a weak signal of a dead item here. Stash→target promotion inserts at the target LRU end (`Front`), so many items that passed the two-hit gate are evicted before they get even one normal-tier hit. Sending them to the stash LRU end makes them the next stash victim. They then need two more stash hits to come back, so items with long reuse gaps are lost instead of kept.
- per-op-time: `get` and `put` p50 did not move, but both p99s went up a little. Likely causes: every normal-tier hit now does an extra write to `lfu_counter`, and the lower hit rate sends more traffic into puts and demotion cascades that touch full stash buckets. This is the least certain part of the reading, because the change itself adds almost no work.

## Takeaway

A "hit during this normal-tier stay" bit is a poor test of liveness, because the stay is short by design: promotion lands at target LRU, and a second miss pushes the item out quickly. Unreused demoted items on hm_0 are long-gap reusers, not two-hit bursts, so giving every demotion stash MRU (as in v002) should stay. Any future demotion-placement signal needs history that outlives a single normal-tier stay, or it has to give items enough normal-tier time to show reuse first.
