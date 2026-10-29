# Variant v002 Insight

## Helped

- per-op-time: Balanced placement ends many demotion chains early. A victim or stash hit that finds a free slot in its other legal bucket is inserted with no eviction and no stash write, so `get` and `put` do less work along both the median and tail paths. This went against the hypothesis, which expected the extra `is_full` checks to make operations slightly slower. The speed-up is much larger than the near-flat hit-rate change would suggest, so treat its size with caution until it is re-measured. It may be partly noise.

## Hurt

- hit-rate: At size 16384, where pressure is highest, the change did almost nothing, because a key's target and probing buckets are nearly always both full and the new free-slot branches rarely fire. At size 131072, there is free space, so the change does fire, but it fills probing slots with cold target-LRU victims (inserted at the Front/LRU end). Those slots are then unavailable to the neighbouring bucket's own keys and to hot target→probing promotions. Placement became more even, but the cache kept colder items in place of hotter ones, which slightly lowered the hit-rate on web_0.

## Takeaway

Filling free slots in the other legal bucket does not help the hit-rate in this Dash segment. When the cache is under pressure, there are no free slots to fill. When there are free slots, the change parks cold evictees in space that hot promotions need. To raise the hit-rate, a placement change must decide what is kept by recency or heat, not just whether a bucket has a free slot. The one clear gain is shorter, cheaper demotion chains, and that latency gain could be kept separately from the placement policy.
