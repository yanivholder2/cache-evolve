# Variant v003 Insight

## Helped

- (none measurable): The stamp fast path in `touch` skips the unlink/push_back for protected nodes relinked within the last `protected_cap / 2` relinks. That removes list-update work per hit, but the fitness vector only has a hit-rate dimension, so the saving is not visible.

## Hurt

- hit-rate: Skipping the relink leaves recently hit nodes at their old protected position. Their effective recency is understated, so the protected LRU end can pick a slightly wrong demotion or eviction victim. The result is a small drop against v001.

## Takeaway

Approximating protected-segment recency with a relink-clock stamp costs a little hit-rate, so the hypothesis's "hit-rate does not drop" condition was not strictly met. The claimed benefit, less work per hit, is not measured by this evaluation. The change can't be judged a win without a per-op time dimension. With hit-rate as the only dimension, the exact-recency parent v001 is the safer choice.
