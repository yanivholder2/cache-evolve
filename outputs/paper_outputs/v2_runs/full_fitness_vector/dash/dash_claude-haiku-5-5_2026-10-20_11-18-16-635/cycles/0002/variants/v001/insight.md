# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The only change is that promotions into a full target bucket now use `InsertPosition::Front`, so those promoted keys become the first eviction candidates, whereas the parent's unconditional `Back` kept them longer.

## Takeaway

Gating the promotion insert position on target-bucket fullness reduced hit rate relative to the parent, and the full-bucket `Front` branch is the only difference from the parent, so it accounts for the loss. Occupancy alone did not separate the effects the hypothesis assumed: promotions into full buckets still need the protection that `Back` gave them on this LIRS workload.
