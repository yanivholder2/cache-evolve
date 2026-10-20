# Variant v002 Insight

## Helped

- **per-op-time**: Reducing sketch depth from 4 to 3 cuts one hash computation per operation, improving both get (132→90 ns) and put (166→117 ns) latency by ~25%.
- **metadata-size**: Sketch heap shrinks from 65536 to 49152 bytes with one fewer independent hash row, a direct benefit of the geometry reduction.

## Hurt

- **hit-rate**: Declined from 0.7362 to 0.7334 as the sketch's accuracy drops with fewer independent hash rows, leading to coarser frequency estimates and slightly worse eviction decisions.

## Takeaway

Reducing sketch depth to 3 delivers meaningful per-operation latency gains and memory savings through reduced hashing work, but trades away a small amount of hit-rate accuracy because fewer hash rows increase collision collateral. This variant confirms that moderate depth reduction (from 4 to 3) balances latency gains against the accuracy cost more favorably than the hypothesis originally expected.
