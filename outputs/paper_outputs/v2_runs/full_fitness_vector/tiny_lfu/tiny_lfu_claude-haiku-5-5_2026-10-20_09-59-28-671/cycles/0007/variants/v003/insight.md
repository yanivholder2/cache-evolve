# Variant v003 Insight

## Helped

- hit-rate: Recovered only a trivial amount of the hit-rate lost by the parent, far short of the restoration the hypothesis targeted. The within-block column derivation from the seeded per-row hash appears to have changed collision spread only marginally.
- metadata-size: The TinyLfuCache struct reports slightly smaller than the parent's, which the hypothesis did not predict (it forbade layout changes). The Node and sketch-heap sizes are unchanged. The cause is not visible from the fitness vector and needs the overlay diff to confirm.

## Hurt

- per-op-time: The get p50 and both get and put p99 moved in the wrong direction relative to the parent, so the one-cache-line grouping from the parent did not deliver a latency gain here. Put p50 is effectively flat. The likely cause is the per-row seeded column lookup adding work on the get path, but this is inferred, not measured from the diff.

## Takeaway

Choosing the block from the key hash and the column from the per-row seed did not restore the hit-rate that the parent's block hash cost, so the collision-spread hypothesis is largely refuted for this change. The variant also gave up the parent's get-path latency benefit, meaning the speedup depended on the simpler column mapping. Note: the variant's source diff could not be read when this insight was written (the overlay path had no readable source files), so the attributions above are inferred from the hypothesis and fitness vectors and should be verified against the diff.
