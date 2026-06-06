# Variant v001 Insight

## Helped

- hit-rate: No dimension moved — halving `bucket_size` and doubling the per-segment bucket count in `layout` (then re-applying the inherited 3:1 split) gave hot keys more addressable target buckets, but that extra addressability rescued no key the deeper-bucket parent was already evicting.

## Hurt

- hit-rate: Trading depth for addressability is a wash here: every slot the narrower buckets remove from a hot bucket's depth is handed back as a new bucket, so the same hot key that overflowed a wide bucket now overflows its narrower share, and the conserved-capacity guard means no net residency was gained.

## Takeaway

Spreading colliding keys across twice as many half-width buckets does not lift hit-rate on top of the 3:1-split parent: under a conserved budget, addressability and depth are interchangeable for this workload because halving bucket width shrinks each bucket's overflow tolerance by exactly the amount the doubled count restores. The "addressability beats depth" lever the cross-cycle insight predicted is, at this geometry, neutral rather than positive — the occupancy ceiling is set by total capacity, not by how it is partitioned between width and count.
