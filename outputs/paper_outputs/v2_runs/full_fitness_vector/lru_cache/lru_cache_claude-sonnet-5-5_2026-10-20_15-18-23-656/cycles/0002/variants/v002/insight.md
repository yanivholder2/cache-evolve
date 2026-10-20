# Variant v002 Insight

## Helped

- metadata-size: Folding the segment flag into a spare bit of a link field removes the separate `seg` field and its alignment padding. The node shrank by far more than one bool would explain, so the layout was probably compacted further. `LruCache` also shrank, consistent with dropping the duplicated segment counters in favour of the existing length fields.

## Hurt

- per-op-time: Every link read and write now goes through a mask or `segment()` / `set_segment()` / `prev_idx()` accessor. This adds work to each get and put, and the put path, which relinks the most, slowed most. Promotion and demotion also cost more than before, although the parent's tail problem was not made worse.

## Takeaway

Packing the segment bit into a link field removed the per-node metadata cost that the parent's segmentation introduced. It did not change the policy, so the scan-resistance behaviour was preserved. The price is a small constant per-operation cost for masking, so this is a space-for-cycles trade that suits memory-bound deployments more than latency-sensitive ones.
