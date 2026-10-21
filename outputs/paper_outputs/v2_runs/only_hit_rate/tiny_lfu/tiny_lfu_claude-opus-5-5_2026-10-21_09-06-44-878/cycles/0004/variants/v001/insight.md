# Variant v001 Insight

## Helped

- hit-rate: The new `pending_miss_hash` makes `put` skip its sketch increment when it replays a `get` that just missed. Before this change, the replay counted each miss twice and each hit once, so the estimate for a key outside the cache rose about twice as fast per access as the estimate for a resident key. That made repeatedly-missing keys look hotter in the strict-`>` contest than residents with the same real access count, which pushed admission toward churn. With miss and hit accesses now weighted equally, sketch estimates track true access counts, and the contest compares newcomers and `select_victim` candidates on equal terms. The gain is real but small.

## Hurt

## Takeaway

The double count in v003 was not a useful admission bias on workload-a. Removing it helped a little, so the "doorkeeper/dedup" question is settled in favour of dedup, but this change is not a major lever. The fix mostly shifted every first access down by one count, which keeps the ranking between a first-time newcomer and a resident accessed once (both still tie and the newcomer is rejected). It only changes outcomes where a key misses repeatedly while competing against residents that collect their counts through hits. The dedup also cuts per-op work: `get` hashes once, and a deduped `put` skips the 4-counter increment. Only hit-rate was measured, so that saving is unscored. Since sketch counts now equal true access counts (up to the remaining collisions), later aging or reset hypotheses can be judged on frequency signals that are no longer skewed by the replay pattern.
