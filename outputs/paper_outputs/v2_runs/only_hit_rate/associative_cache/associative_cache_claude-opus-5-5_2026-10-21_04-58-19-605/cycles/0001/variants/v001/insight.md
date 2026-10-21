# Variant v001 Insight

## Helped

- hit-rate: New keys now enter each 16-slot bucket halfway down instead of at the MRU tail, so a key that is never touched again only lasts about half a bucket's lifetime. Keys that do get a second hit still move to the tail, so reused keys survive bursts of one-off and scan keys that would have flushed the whole bucket under plain LRU.

## Hurt

(None measured. This evaluation recorded only hit-rate. per-op-time and metadata-size were not re-measured, so we have no evidence on them either way. The code change does not add any per-Item fields; it adds one `bool` per bucket. The miss path now uses `Vec::insert` at the midpoint instead of `push`, which shifts about half the bucket on every miss.)

## Takeaway

On workload-a, the bucket's eviction order isn't what limits the baseline. Where new keys enter the bucket is: making them earn MRU protection with a second hit, instead of granting it on insert, stops one-off keys from washing out reused ones. Changing only the insert position and leaving hits and eviction alone was enough to gain hit rate. That makes the insert depth (`INSERT_DEPTH_NUM/DEN`) the obvious next knob to tune. It also means per-op-time needs measuring before this counts as a free win, because of the extra element shifts on each miss.
