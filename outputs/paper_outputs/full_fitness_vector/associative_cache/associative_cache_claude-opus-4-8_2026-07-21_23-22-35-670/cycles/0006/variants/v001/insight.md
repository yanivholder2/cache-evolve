# Variant v001 Insight

## Helped

- metadata-size: Dropping the unused `lfu_counter` and `timestamp` fields shrinks `Item<u64,u64>` back to just `{key, value}`, reclaiming the full metadata floor the SLRU bucket never used.
- per-op-time: A smaller `Item` makes the bucket's `insert(0, …)`/`remove(0)` shifts and single-element rotations copy fewer bytes, halving put's p50.

## Hurt

## Takeaway

The parent's segmented bucket read `protected_len` and rotations only — never `lfu_counter` or `timestamp` — so deleting those two `Item` fields was behaviourally inert: hit-rate held exactly while metadata-size fell 40→16. This confirms the cycle-5 40-byte figure was pure overlay-hygiene debt (a bucket-only overlay silently reverting `Item`), not a cost of segmentation, and the trim also incidentally speeds the byte-shifting put path.
