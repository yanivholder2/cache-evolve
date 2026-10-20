# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Cutting `SKETCH_DEPTH` from 4 to 2 with the width factor held at 8 leaves each row just as wide, but `estimate()` takes the minimum across only two rows, so fewer independent rows filter out collision over-counts and the `incoming_freq > victim_freq` admission check is made on noisier frequencies.

## Takeaway

The hypothesis treated sketch width as the thing that controls collision error, but this result shows that the row count is what lets the min-across-rows estimator suppress collisions. Halving depth gave back the hit-rate that the parent's width change had gained, and the memory saving was never measured here because this variant's fitness vector has no metadata dimension, so the trade-off stays unresolved and depth 4 with width factor 8 remains the parent's configuration.
