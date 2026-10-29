# Variant v002 Insight

## Helped

- hit-rate: Demoted items with a nonzero `lfu_counter` (checked before halving) now land at the MRU (`Back`) end of the stash or their true target. A key that was reused but lost its slot to a cascade now outlives the cold evictees around it, so it is more likely to still be cached at its next access. The gain is small. It is largest on web_0 at the large cache size, smaller on the two small-cache cells, and absent on hm_0 at the large size, where v003's gate had already protected the reused working set.
- per-op-time (get p99): Target → probing promotion now also requires `lfu_counter >= 2`, so a demotee landing on the "hottest" index is not promoted again on its very next hit. That removes some `remove_at` + `promote_to_probing` + cascade chains, and the get tail improved slightly instead of showing v001's churn.

## Hurt

- per-op-time (get median, the summary statistic): Every target hit now reads the counter and runs an extra threshold comparison before the promotion decision. Items that used to move up to probing now stay in target and take the `get_from_position` touch. This adds a little fixed cost to the common get path on top of v003's per-hit increment.
- per-op-time (put): Puts can trigger demotion cascades, and on that path `demote_from_normal_bucket` now computes a landing position for each item (twice when there is a cascade) and can insert at `Back`. In the stash, a demotee at the MRU end means the next put evicts colder stash entries instead of the demotee, so the demotion and eviction work during puts goes up a little.

## Takeaway

Choosing the demotion landing spot from reuse evidence does stack on v003's stash gate without bringing back v001's promotion churn, but only because the probing promotion also needs a count of 2. Without that guard, MRU landing would feed the hottest-index promotion check directly. The hit-rate gain is marginal and shows up only where the gate had not already saturated the benefit, while each get on the summary path pays a small extra cost. So "protect reused demotees" looks mostly redundant with the gate, not a separate source of gains.
