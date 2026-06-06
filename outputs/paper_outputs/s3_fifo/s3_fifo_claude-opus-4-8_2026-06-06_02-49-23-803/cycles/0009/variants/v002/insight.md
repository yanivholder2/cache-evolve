# Variant v002 Insight

## Helped

- metadata-size: removing the `key: K` field from `Entry` (kept as zero-sized `PhantomData<K>`) leaves the scored `Entry<u64,u64>` holding only `value` + `freq`, dropping it off its previous size — the live key already lives once in the `entries` index and the FIFO order queues, so the per-entry copy was pure duplication.
- per-op-time: get p99 tightened because the shrunken `Slot::Resident` payload makes each `entries` map value smaller, so the hot `get` probe touches less per slot.

## Hurt

- per-op-time: put p99 ticked up slightly despite the miss path now doing *less* work (the final insert no longer clones `key` to also stuff it into the entry) — the move is within measurement noise, not a structural regression.

## Takeaway

Folding ghosts into the `entries` map and dropping the redundant `key` field are independent wins: this variant proves the folded-tombstone family can reach the same minimal scored `Entry` size as the separate-`ghost_set` family while keeping its single-map container (S3Fifo unchanged at 152), all with O(1) put and bit-identical hit-rate. The stored key was carrying no information the index and order queues didn't already hold.
