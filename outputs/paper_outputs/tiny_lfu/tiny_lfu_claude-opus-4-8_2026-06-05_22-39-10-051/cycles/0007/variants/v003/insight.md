# Variant v003 Insight

## Helped

- (none): the only measured dimension is hit-rate; the intended memory/cost win from the 4x-narrower sketch is real in principle but is not a tracked fitness dimension, so nothing scored a gain here.

## Hurt

- hit-rate: dropping `SKETCH_WIDTH_FACTOR` from 4 to 1 made the rows 4x more collision-prone, and conservative update — incrementing only the rows currently at the per-key minimum — could not fully recover the accuracy lost to those collisions, so admission decisions degraded.

## Takeaway

On this trace, conservative update does not hold accuracy at 1/4 the sketch width: the narrower sketch produced a measurable hit-rate loss rather than holding within noise. This refutes the hypothesis that CU can compensate for the narrowing and bounds the question from prior cycles — the extra width was not pure waste; this workload genuinely needs more cells than capacity×1 to keep its distinct keys separated.
