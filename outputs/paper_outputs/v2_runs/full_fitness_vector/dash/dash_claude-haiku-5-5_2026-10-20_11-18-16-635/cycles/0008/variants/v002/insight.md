# Variant v002 Insight

## Helped

- metadata-size: Packing the item record into the cycle 0007 v003 `Item<u64,u64>` layout, with the access counter moved into the packed slot, removes the per-item overhead the parent carried, which is the one target H02 set out to move.

## Hurt

- per-op-time (get): The access counter is now packed, and the hit increment that runs on every get hit has to read-modify-write that packed field. The hypothesis kept the counter off the get path, but the hit increment is on it, so the packed write lands on lookups.
- per-op-time (put): Eviction now scans packed counters across the bucket and insert writes the packed record, so every put that touches a full bucket pays for decoding on top of the full-scan comparison.

## Takeaway

Packing the counter recovered the metadata saving without losing eviction information, since hit rate held exactly where the parent sat. The cost moved onto the hit path: the counter increment on get is the one place where the packed layout and the replacement policy share work, so the two changes are not as separable as the rationale assumed. A follow-up should keep the counter in an unpacked slot for the hit increment and pack only the cold fields. This reading is inferred from the hypothesis's design and the measured dimensions; I did not verify it against the overlay diff.
