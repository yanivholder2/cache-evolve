# Variant v002 Insight

## Helped

- per-op-time: Replacing `realign_reuse`'s key-matched rebuild with an index-preserving `reuse.remove(position)` + `insert(index, bit)` that mirrors the policy's single-element move collapsed both p99 tails and dropped the `put` p50 — the O(n²) per-hit rebuild the parent paid is gone, leaving only O(bucket) shifts.

## Hurt

## Takeaway

The parent's blown-out p99 was purely the cost of rebuilding the reuse vector by key-search on every hit, not anything intrinsic to carrying scan-resistance off the item: swapping the bit in place at the same index the policy relocated the item recovers clean tails while `reuse[i] ↔ items[i]`, hit-rate (0.7267), and the 16-byte `Item` floor all hold bit-for-bit. Mirroring the policy's own relocation instead of reconstructing from a snapshot is what makes the scan-resistance state free on time as well as metadata.
