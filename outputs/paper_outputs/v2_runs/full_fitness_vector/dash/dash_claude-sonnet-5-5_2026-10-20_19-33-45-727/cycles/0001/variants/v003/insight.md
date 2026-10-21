# Variant v003 Insight

## Helped

- hit-rate: Requiring a second stash hit before promotion (first hit only refreshes recency in place) keeps one-hit-wonders out of the target buckets, so they no longer displace reused items.
- per-op-time: The first stash hit now skips `remove_at` and `promote_to_target`, so the common get path does less shifting and bucket mutation. Both get and put latencies dropped.

## Hurt

- metadata-size: The new `hit: bool` field did not fit in existing padding as the hypothesis hoped, so `Item<u64,u64>` grew by a full alignment unit (40 to 48 bytes). That is a per-item cost across the whole cache.

## Takeaway

Delaying promotion until a second hit is a cheap, effective admission filter: it improved hit-rate and reduced get time. But a per-item flag is not free, because the struct had no spare padding. A later variant could keep the same filter without growing `Item`, for example by packing the flag into an existing field's spare bits or by encoding it in bucket position.
