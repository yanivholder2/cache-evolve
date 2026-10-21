# Variant v003 Insight

## Helped

- (none): No measured dimension improved over the parent.

## Hurt

- hit-rate: Swapping the hottest probing hit back to the target bucket's MRU end pushes the target's coldest item into the probing bucket, and that item can cascade on to stash. The data shows this displacement costs more than the promotion gains. Baseline already leaves the item resident in probing, where hot items are retained, so the swap adds churn rather than recovering lost hits.
- metadata-size / per-op-time: These dimensions are absent from this variant's fitness vector, so no effect can be claimed. The probing-hit path now does a remove plus up to three `put_at` cascades instead of a plain touch, so get latency would be expected to rise.

## Takeaway

Making promotion bidirectional (probing → target) did not pay off. Moving an item that is already hottest in its tier does not make it more likely to hit, but it forces a cold target item down a demotion chain toward stash and eviction. Keeping hot items in place within the probing tier is as good as or better than relocating them to their home bucket.
