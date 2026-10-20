# Variant v003 Insight

## Helped

- per-op-time: Get and put p50 both fell clearly below the parent's. This matches the hypothesis's intent: packing all rows of one key's counters into a single 64-byte block means one cache line serves every row access per operation.

## Hurt

- hit-rate: Hit-rate dropped outside the tolerance the hypothesis set (within 0.001 of the parent). The block-selection change alters which keys collide in the sketch, even though admission logic is unchanged, so the frequency estimates shifted enough to cost hits.
- metadata-size: The reported TinyLfuCache size grew slightly, although the hypothesis forbade changing cache struct fields and the Node and sketch-heap sizes are unchanged. The cause cannot be pinned down from the fitness vector alone and needs a look at the overlay diff.

## Takeaway

Grouping a key's sketch counters into one cache-line block buys a clear latency win on both get and put, but the block-selection hash also changes collision patterns enough to lose hit-rate. The speed gain is the part worth keeping. Any follow-up needs either a collision-preserving block selection or a compensating change to counter width or aging, not a further cut to the sketch's per-row work. Note: the variant's source diff was not readable when this insight was written, so the attributions above are inferred from the hypothesis and fitness vectors.
