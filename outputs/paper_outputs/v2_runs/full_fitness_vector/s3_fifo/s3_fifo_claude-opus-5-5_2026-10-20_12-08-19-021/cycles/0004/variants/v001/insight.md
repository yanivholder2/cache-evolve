# Variant v001 Insight

## Helped

- hit-rate: The only policy change is the ghost ring length. On the eval size (capacity 2048, initial main 1536) the ring drops from 1536 to 768 fingerprints, so the ghost remembers only the last 768 small-queue demotions. Fewer re-puts land as ghost hits, so the "ghost hit → shrink small" rule fires less often, and `small_size` is pushed upward mainly by the grow-small graduate rule. The move was in the direction H01 predicted, but it was barely above zero. Halving the ghost changed the outcome far less than the ring-hole change did in the parent. The adaptive split seems to absorb most of the change in ghost length, and the `capacity/16` and `capacity/2` clamps leave little room for that shift to matter.

## Hurt

- per-op-time (`get`, the summary): `get` is byte-identical to the parent's: one `index` probe, one slab access, a freq bump. It never touches the ghost. The regression therefore cannot come from the ghost-length change. It points to a benchmark-environment shift or a different code layout, not to anything this hypothesis did. Treat this number as untrustworthy until it is re-measured next to the parent.
- per-op-time (`put`): A shorter ghost should make `put` slightly *cheaper* (the ring and `ghost_map` are half the size and fit better in cache), not slower. `put` regressed by about the same proportion as `get`, which again points to measurement noise, not to the ring change.

## Takeaway

Ghost length on its own is a weak lever here. Halving the ring moved hit-rate only marginally in the predicted direction, so the parent's ghost length is not far from the optimum. Shortening it more is unlikely to pay off unless the adaptive clamps are changed too. The per-op-time regression comes with no code change on the `get` path, so this variant's latency should be re-benchmarked against v002 before anyone concludes that a smaller ghost costs speed. Metadata-size did not move because only the ghost's heap allocation shrank, and that dimension does not measure it.
