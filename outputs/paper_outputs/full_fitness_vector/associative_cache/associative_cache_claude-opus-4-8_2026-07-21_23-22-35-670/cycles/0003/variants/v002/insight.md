# Variant v002 Insight

## Helped

- (none): The `min_by_key(lfu_counter)` tiebreak on fully-protected buckets was the whole thesis, and no dimension moved in its favour — the probed "headroom beyond binary protection" did not materialise.

## Hurt

- hit-rate: Replacing the fully-protected fallback (return front, index 0) with "evict the minimum-`lfu_counter` item" evicted a hot-but-least-reused entry instead of the oldest one, and on this trace that victim was reused sooner than the LRU front would have been — hit rate dropped below the parent rather than rising.
- metadata-size: Regressed from the parent's 24 bytes to 40 because the overlay carried forward only `eviction_policy.rs` and omitted the parent's slimmed `item.rs`/`bucket.rs`, so `Item<u64,u64>` reverted to the fatter base layout — an unintended loss unrelated to the hypothesis's logic change.

## Takeaway

Using the already-maintained `lfu_counter` as an LFU tiebreak on saturated buckets is not free hit-rate headroom: least-frequently-reused is a worse eviction signal than plain LRU-front here, refuting the "beyond binary protection" question at zero benefit. The change also silently forfeited the parent's metadata win by not re-applying its slimmed `Item`, so the variant regressed on two axes while touching only the eviction victim choice.
