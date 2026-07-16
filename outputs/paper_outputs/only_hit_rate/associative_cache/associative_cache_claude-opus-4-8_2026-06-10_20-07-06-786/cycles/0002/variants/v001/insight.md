# Variant v001 Insight

## Helped

- (none): The hypothesized composition produced no measured gain on any dimension; front admission did not add to v002's frequency-first protection.

## Hurt

- hit-rate: Inserting new keys at the front (`insert(0, item)`) with `lfu_counter = 0` makes a brand-new key simultaneously the front-of-tie and minimum-frequency entry, so under `min_by_key` ties it becomes the *immediate* eviction victim — destroying the recency protection v002's back-push gave fresh entries and evicting genuinely-reused keys before they can earn their first hit.
- per-op-time: `insert(0, …)` shifts the whole bucket vector on every new-key admission, an O(bucket_size) cost the trait-default back-push avoided.

## Takeaway

The two scan-resistance levers do not compose — they conflict. v002's frequency-first eviction already protected hot keys; front admission then strips the front-biased tie-break that let new keys survive long enough to be re-referenced, so the bucket discards rising keys at probation instead of one-hit-wonders. Stacking probationary front admission onto frequency-first victim selection over-penalizes new entries and lowers hit rate below the parent rather than lifting it.
