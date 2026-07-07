# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Scanning for the first post-decay cold item (counter == 0) can select a higher-index item that was inserted more recently under the mid-point scheme, whereas the parent's unconditional removal of index 0 always targets the oldest-inserted item — a better warm-lock eviction target when recency correlates with position.

## Takeaway

Replacing the blind `remove_at(0)` with a post-decay cold scan did not help: in the mid-point insertion layout, index 0 is the oldest item in the bucket, so it already functions as a reasonable LRU-style target during warm-lock resolution. The cold scan instead risks evicting a more recently inserted item whose counter happened to fall to zero first, undermining the recency signal embedded in insertion order. The warm-lock path is either too infrequent on this trace or its per-event eviction quality too coarse for the principled cold-first alignment to outweigh the recency cost.
