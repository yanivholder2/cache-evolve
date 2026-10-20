# Variant v002 Insight

Note: I could not list or read the overlay source in this session. This insight comes from the hypothesis and the two fitness vectors, not from a code diff.

## Helped

- get per-op-time: The `get` hot path was nominally unchanged, and the hypothesis kept it untouched on purpose. Any gain is within run-to-run noise, not a result of the change.

## Hurt

- per-op-time (put): `put` latency, especially the tail, got worse. The change moved hash recomputation onto the eviction path, and eviction runs inside `put`. Each eviction now pays for an extra hash of the victim key to get its sketch estimate.
- metadata-size: The hypothesis targeted a smaller `Node`, but the `Node` size did not change. The `Node` type, the sketch heap and the cache struct all kept the same size. Either no removable field existed or the removal did not reduce size.

## Takeaway

Removing a cached field from `Node` did not reduce its size. Padding or alignment of the remaining fields probably absorbed the saving, so trimming a field only helps if it crosses an alignment boundary. Moving work onto the eviction path to save space also taxed `put` latency. Hit-rate stayed identical, so admission semantics were preserved, but the change gave no footprint gain to offset the cost.
