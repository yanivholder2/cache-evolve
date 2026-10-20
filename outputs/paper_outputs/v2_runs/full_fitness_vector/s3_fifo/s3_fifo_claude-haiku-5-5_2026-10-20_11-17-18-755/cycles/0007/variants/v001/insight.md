# Variant v001 Insight

## Helped

- metadata-size: Moving `freq` out of `Entry` and into the per-key index record removes the 8-byte-per-entry cost that the parent carried, so `Entry` is now the bare `{ key, value }` pair.
- get: The smaller `Entry` means each queue slot is more compact, so the value read at `head - id` touches less memory on the hit path, while the single index lookup per hit is unchanged.
- put: Resident-key updates write only the value through the queue and never touch `freq`, so the write path does less work than the parent, which had to maintain the counter inside the entry.

## Hurt

- None measured. The frequency logic moved into the index record, but the container-level size did not grow in the measurement, which suggests the extra byte in the index record was absorbed by padding or is not counted in the reported size.

## Takeaway

Relocating the `u8` saturating frequency counter from the queue entry to the index record cut per-entry metadata without changing hit rate, since the eviction, ghost, and promotion paths still read the same counter value. The change works because the index record already existed for lookup, so the counter rides along with a structure that was paid for anyway. Confirm the container-size result against the reported `S3Fifo` figure before treating the one-byte index growth as free.
