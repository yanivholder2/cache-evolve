# Variant v001 Insight

## Helped

## Hurt

- per-op-time (put): The ghost queue now carries a `(K, u64)` sequence tag and a `HashMap<K, u64>` index, so every eviction into ghost does a map insert plus a key clone and a queue push, and overflow goes through a tombstone-skipping pop. That per-eviction bookkeeping lands on the put miss path, and it outweighs the position scan removed on ghost hits, which is the only path that scan ran on, so put latency rose at both p50 and p99.
- metadata-size: The `S3Fifo` struct gains a `ghost_seq: u64` counter field, which is the only struct-level change, so the struct footprint grows while `Entry` stays the same size.

## Takeaway

Sequence-numbering the ghost queue did keep the eviction policy intact (hit-rate matches the parent exactly), but it did not reduce put cost. The O(n) `position` removal it targeted only ran on ghost hits, so it was not the dominant put-path cost that H01 assumed, and the lazy-tombstone scheme added work to every ghost insert. The cost of the put miss path is likely elsewhere, so a follow-up should profile that path before restructuring the ghost queue again.
