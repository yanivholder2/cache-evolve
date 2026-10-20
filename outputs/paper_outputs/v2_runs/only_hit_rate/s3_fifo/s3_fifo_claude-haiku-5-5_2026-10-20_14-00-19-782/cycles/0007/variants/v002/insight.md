# Variant v002 Insight

## Helped

## Hurt

## Takeaway

H02 deletes the `ghost_set` HashSet mirror and makes ghost membership a linear scan of the ghost VecDeque, while the eviction and promotion logic stays byte-for-byte equivalent to v001. The fitness vector reports only hit-rate, which did not move, as expected from an equivalence-preserving change; the claimed metadata-size reduction and the per-op-time cost of the O(main_size) miss-path scan are not measured here, so neither direction is confirmed.
