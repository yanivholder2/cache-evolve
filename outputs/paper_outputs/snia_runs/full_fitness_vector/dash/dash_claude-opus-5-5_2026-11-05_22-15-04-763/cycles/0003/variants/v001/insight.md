# Variant v001 Insight

## Helped

- hit-rate: Unchanged on every trace and size, which matches the hypothesis. Moving the stash block after the probing block leaves every block body as it was. Because a key is resident in at most one slot, the reorder changes no placement, marking or promotion decision. It is a pure lookup-cost experiment.
- per-op-time: `put` p99 went down even though nothing on the `put` path changed. Only `promotion_get` was reordered. Treat this as benchmark noise or a side effect, not a result of the hypothesis.

## Hurt

- per-op-time: `get` p50 (the summary statistic) and `get` p99 both got worse, the opposite of the prediction. With target → probing → stash, every stash hit now first scans two normal-tier buckets. That includes the first-hit mark that every new key needs and the second-hit promotion. The time this added to stash hits was more than the time the reorder saved on target and probing hits.

## Takeaway

The hypothesis assumed two things: that under the two-hit gate most `get` hits land on normal-tier residents, and that the stash-bucket scan dominates their cost. The regression shows that at least one of these is false. Stash hits are a large enough share of hits, or the stash `get_position` scan is cheap enough, that checking stash first was the better order. v003's stash-first lookup did not cause the gated variants' slower `get` compared with the baseline. Future work on that slowdown should look at the work done per hit, such as the hottest→probing promotion on target hits or the cost of the stash mark/promote path. Lookup order is not the lever.
