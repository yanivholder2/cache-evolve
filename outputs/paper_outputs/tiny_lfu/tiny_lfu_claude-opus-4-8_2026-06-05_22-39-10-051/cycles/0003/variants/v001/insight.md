# Variant v001 Insight

## Helped

- hit-rate: Pushing `SKETCH_WIDTH_FACTOR` 16→64 (width 32768→131072 at capacity 2048) shaves a few residual estimator collisions, nudging hit-rate up by a hair — the admission gate `incoming_freq > victim_freq` misjudges marginally fewer keys.

## Hurt

- (none observed): No competing dimension is in the vector, so the 4x larger sketch allocation carries no measured penalty; hit-rate is the only signal and it did not regress.

## Takeaway

This 4x width step localizes the fidelity plateau: with `SKETCH_DEPTH` and every policy mechanic byte-for-byte unchanged, the count-min cells already separate the trace's distinct keys well enough at factor 16, so quadrupling width again buys a negligible hit-rate gain. The estimator-width lever that paid off 4→16 has essentially flattened — further accuracy must come from a different axis, not more sketch memory.
