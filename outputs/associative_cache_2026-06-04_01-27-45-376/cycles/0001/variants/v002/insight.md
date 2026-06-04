# Variant v002 Insight

## Helped

- metadata-size: `Item<K,V>` layout was untouched — the CLOCK reference bits live in a `RefCell<Vec<bool>>` side-channel on `ClassicLRU` (forced by the trait taking `&[Item]` on the eviction path), so per-item bytes stay at the baseline's 40.

## Hurt

- hit-rate: CLOCK's single reference bit collapses LRU's full recency ordering into a two-level "hot/cold" partition, so frequently-touched items that fall behind the sweep hand are evicted before genuinely cold neighbours — the workload-a hit rate drops ~3.8 points vs. the move-to-back baseline.
- hit-rate: `Bucket::remove_at` deletes a middle item without notifying the policy, leaving a stale reference bit attached to the item that shifted into that slot; the doc-comment accepts this as an "approximation cost" but it further blurs which slot is actually hot.

## Takeaway

For Dash buckets of size ~16, swapping LRU's bucket-internal reordering for CLOCK in the in-bucket policy slot is a hit-rate regression with no measurable metadata savings — the bucket is too small for CLOCK's coarse recency signal to recover what `Vec::remove`+`push` captures exactly. Any future per-op-time win from skipping the shift has to be weighed against this hit-rate floor, and the trait's `&[Item]` eviction signature is what forces the side-channel design in the first place — touching that signature is a prerequisite for an Item-resident reference bit.
