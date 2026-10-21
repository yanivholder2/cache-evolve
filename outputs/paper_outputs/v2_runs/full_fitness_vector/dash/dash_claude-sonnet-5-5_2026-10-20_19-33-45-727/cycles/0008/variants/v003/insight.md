# Variant v003 Insight

## Helped

- per-op-time: Most first-sighting puts now return after one table write, with no insert and no eviction. That removes the costly path from put. The get path also looks faster, probably because fewer inserts and evictions disturb the segment.

## Hurt

- hit-rate: The 256-slot direct-mapped table overwrites on collision, so a key's first sighting is often lost before its second put arrives. Those keys are then admitted late or never. Every first-time key is also denied admission, which costs compulsory misses on keys that would have hit.

## Takeaway

A small fingerprint doorkeeper does make puts cheap, because it skips insert and eviction work on one-hit keys. At this workload's scale, though, a 256-entry table without aging is too lossy to keep hit-rate. Per-segment capacity and the overwrite policy matter more than fingerprint width. Metadata size is unchanged because the table sits in the segment struct, not the per-item struct.
