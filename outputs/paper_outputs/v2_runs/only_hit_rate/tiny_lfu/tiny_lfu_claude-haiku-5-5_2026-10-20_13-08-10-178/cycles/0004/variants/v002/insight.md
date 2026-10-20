# Variant v002 Insight

## Helped

- hit-rate: Doubling `SKETCH_WIDTH_FACTOR` from 4 to 8 doubles the counters per row of the count-min sketch, so `estimate()` over-counts less from colliding keys and the admission check in `put()` compares frequencies that are closer to the true values, which is the mechanism the hypothesis proposed.

## Hurt

## Takeaway

Widening the sketch is a lever independent of the decay and key-mapping changes tried earlier: the gain on workload-a at size 2048 is small but in the predicted direction, which suggests counter collisions were contributing to admission errors. This variant's fitness vector reports only hit-rate, so the metadata cost the hypothesis predicted (the sketch's heap size scales linearly with the width factor) is not confirmed here and should be checked against a metadata-size measurement before this change is kept.
