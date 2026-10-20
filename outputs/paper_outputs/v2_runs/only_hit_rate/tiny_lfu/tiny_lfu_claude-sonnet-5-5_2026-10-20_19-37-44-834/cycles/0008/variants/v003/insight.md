# Variant v003 Insight

## Helped

- hit-rate: Changing `SKETCH_WIDTH_FACTOR` from 4 to 8 doubles the counters per row. That should cut collisions between hot and cold keys in the frequency sketch, so admission decisions are slightly better. The gain over the baseline is small.

## Hurt

- metadata-size: v003's fitness vector has no metadata-size measurement, so it can't be compared with the baseline. The hypothesis said this change would halve sketch metadata. But going from 4 to 8 is an increase in width. Unless the factor is defined inversely, the sketch should grow, not shrink.
- per-op-time: Not measured for v003, so no effect can be attributed. A wider sketch could add cache misses on the sketch lookup, but this is unverified.

## Takeaway

The hypothesis framed 4 to 8 as a "reduction" that saves metadata, which contradicts the direction of the edit. The evaluation only recorded hit-rate, so the metadata-per-hit-rate trade-off it targeted was never tested. The small hit-rate gain suggests the baseline's sketch width isn't a major bottleneck on this trace. Judging this change needs metadata-size and per-op-time measured for v003. The hypothesis should also state which direction the factor scales the sketch.
