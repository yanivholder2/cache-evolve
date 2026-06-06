# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Crediting `lfu_counter` on the stash-overflow second-chance path produced no measurable change in hit-rate, despite being structurally symmetric with the stash-hit credit that worked in the parent. The asymmetry is in what the two paths select: the stash-hit path fires for items that just received a genuine access (demonstrably live), while the stash-overflow second-chance path fires for the LRU victim in an overflowing stash bucket — the coldest, least-recently-used resident with no recency evidence at all. Bumping the counter on these cold LRU evictions does not reduce their eviction risk in the target tier in a meaningful way, because on this workload the target tier is not so congested with cold newcomers that a single counter credit changes eviction ordering. The protection gap on the overflow path either does not fire frequently enough or does not involve items hot enough to affect the measured hit-rate.
