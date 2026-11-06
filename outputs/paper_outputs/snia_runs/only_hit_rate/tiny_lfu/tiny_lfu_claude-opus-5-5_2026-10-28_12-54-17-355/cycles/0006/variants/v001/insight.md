# Variant v001 Insight

## Helped

- hit-rate (web_0 small): counting the get-miss and the following put as one access stops every one-time key from starting at an estimate of 2. At the smallest sketch width, those extra increments were adding collision noise. With them gone, the contest against the sampled probation minimum separates real repeat keys from one-hit keys a little more cleanly. The gain is only a fraction of what cycle-0005 v001 got from the same change on top of its own parent. So the two mechanisms overlap rather than add: v003's coldest-of-4 victim already captures much of the same "drop the truly cold resident" benefit.

## Hurt

- hit-rate (web_0 large): this cell took the largest regression. The sketch width scales with capacity (4× capacity, rounded up to a power of two), so at 131072 collision noise was already low and the change had little noise to remove. What it did remove was the extra increment every new arrival got on insert. The sketch never ages, so that extra count had been working as a small recency boost. It let freshly inserted window candidates beat probation residents whose counts came from old history. Without it, the strict `>` contest leans further toward stale frequency.
- hit-rate (hm_0, both sizes): small losses at both sizes. The recency-driven hm_0 workload depends most on new arrivals getting admitted, and it loses the same insert-time boost. The sampled victim softened the hm_0-small loss that single counting caused under cycle-0005 v001, but did not cancel it. As the parent insight noted, the sample only reaches the probation head, not the newly admitted entries at the tail.

## Takeaway

Single-count sketch accounting and the sampled probation victim are not additive. Both make admission decisions favour accumulated frequency over fresh arrivals, so stacking them gives only a small extra web_0-small gain and costs more at the large size and on hm_0. In this lineage, the "inflated" estimate of 2 on miss-then-insert was not just noise. In a sketch with no aging, it was the only recency weight a newcomer had. Removing it should be paired with an explicit recency or aging mechanism (e.g. periodic sketch halving), not done alone.
