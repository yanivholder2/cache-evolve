# Variant v002 Insight

## Helped

- metadata_size: held flat — the frequency-aware victim reused the
  pre-existing `lfu_counter` field on `Item`, so adding a reuse-protected
  eviction rule cost zero new per-item bytes.

## Hurt

- per_op_time: get latency edged up because `get_from_position` now writes
  `items[position].lfu_counter += 1` on every in-place hit, and full-bucket
  eviction replaced the O(1) constant-`0` victim pick with an O(n) scan over
  the bucket for the smallest `lfu_counter`.

## Takeaway

This variant proves the eviction-victim slot is an independent lever from
promotion in a Dash bucket: it kept promotion strictly recency-based (MRU
move-to-back) while switching *who leaves* to the least-reused resident,
yielding a recency-promote / frequency-evict hybrid. The frequency victim is
free on metadata (the counter already existed) but not on time — it adds a
per-hit counter write and a linear full-bucket victim scan. Whether protecting
recurring loop keys actually lifts hit rate can't be judged here, since the
parent baseline was never measured on the targeted `lirs:multi1` scan/loop
trace.
