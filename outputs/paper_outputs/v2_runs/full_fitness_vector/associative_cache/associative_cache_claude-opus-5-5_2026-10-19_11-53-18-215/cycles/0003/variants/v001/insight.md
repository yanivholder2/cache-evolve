# Variant v001 Insight

## Helped

- metadata-size: `Item<u64,u64>` shrank to the size the hypothesis targeted. The 16-byte `Instant` became a `u32` logical tick and the `usize` LFU counter became a `u32`. With the field order `key, value, timestamp, lfu_counter`, the two `u32`s fill one 8-byte word with no padding. The `AssociativeCache` handle did not change, as expected.
- per-op-time: get and put both got faster at p50 and p99. Put gained the most. Two separate effects explain this:
  - Each SLRU memmove now moves fewer bytes per slot. That covers the probation-hit `rotate_left(1)` across the protected segment, the full-bucket `remove_at(0)` and the `insert(len0, ..)` shift. v002's insight named these moves as its latency cost.
  - `Item::new` now increments a thread-local `Cell<u32>` instead of calling `Instant::now()`. That takes a wall-clock read off every miss/insert path. This is the likely reason put improved more than get.

## Hurt

## Takeaway

On the merged-bucket SLRU, the `Item` timestamp and LFU fields are dead weight for replacement decisions. The SLRU path never reads them, so shrinking them cost nothing in hit rate. The hit rate matched v002 exactly, which confirms the two cycle-0002 changes really are independent. The latency gain shows that v002's slowdown was partly about slot width, not only about how far the shifts reach. A narrower `Item` (or one with no `timestamp`/`lfu_counter` at all when `slru == true`) directly cuts what every SLRU promotion and eviction costs. Removing the per-insert `Instant::now()` was a meaningful hidden saving on put as well.
