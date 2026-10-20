# Variant v002 Insight

## Helped

- per-op-time: The put path is the one the hypothesis targeted; ghost entries stored as 64-bit fingerprints instead of cloned keys remove the per-eviction key clone on put, and the put median and tail both dropped against the parent.

## Hurt

- metadata-size: The `S3Fifo` struct grew against the parent because the ghost now carries a hash set of fingerprints alongside the queue, which is the extra bookkeeping needed for O(1) ghost membership. The `Entry` size is unchanged, so the cost sits in the container, not the stored entries.

## Takeaway

Swapping ghost keys for fingerprints did what the hypothesis predicted on the put side and left hit rate unchanged, which fits a fingerprint-only change to the ghost. The cost is the added hash-set state in the queue struct. The get-path improvement is not credited to this change, since the hypothesis left `get()` untouched, and I could not read the overlay source to confirm the diff matches the stated plan, so these conclusions rest on the fitness vectors and hypothesis alone.
