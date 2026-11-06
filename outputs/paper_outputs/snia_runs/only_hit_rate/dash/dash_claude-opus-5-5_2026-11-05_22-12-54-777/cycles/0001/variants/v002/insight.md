# Variant v002 Insight

## Helped

- hit-rate: Changing each segment's split from 8 normal / 24 stash to 24 normal / 8 stash, at the same total capacity, raised hit-rate at both sizes on both MSR traces (web_0, hm_0). With a larger target/probing tier, reused blocks that were promoted out of stash are evicted less often by later promotions.

## Hurt

(none measured. v002's fitness vector reports only hit-rate. metadata-size and per-op-time were not re-measured. Since only the bucket counts changed, neither should move, but this run did not check that.)

## Takeaway

Making the protected normal tier 3/4 of each segment's buckets is a clean improvement that changes nothing else. It works because the admission path is "every new item goes to stash," so a small normal tier was the bottleneck for keeping the reused set. The hypothesis expected the biggest gain at size 16384, but the result went the other way. Gains were clearly larger at size 131072 and small at 16384, most of all on hm_0. At the small size the cache is too tight for any split to hold the working set. So the gain grows when the larger normal tier can hold most of the reused blocks, and the smaller stash is still big enough to filter one-touch scans. Child variants should sweep the ratio further (e.g. 7/8 normal), or pair this topology with admission or promotion changes. They should not expect topology alone to fix the small-cache case.
