# Variant v003 Insight

## Helped

- metadata-size: As predicted, doubling `SKETCH_WIDTH_FACTOR` (16→32) only grew the off-summary sketch heap; the scored per-node footprint stays node-bound at 24 B, so the width axis is invisible to this dimension.

## Hurt

- per-op-time: The wider table doubled the sketch heap, lengthening the `position_for` double-hash stride across more cache lines and raising both `get` and `put` p50/p99 — the larger table is touched on every read, so the locality tax is unconditional.

## Takeaway

Sketch width is saturated at factor 16: doubling to 32 left hit-rate flat (a hair lower, within noise) while strictly worsening per-op time. The standing question — whether collisions still carry recoverable accuracy past current width — is answered negatively; aliasing recovery has plateaued, and further width only buys cache-line cost, so the hit-rate lever lives elsewhere (estimate rule or admission), not in more columns.
