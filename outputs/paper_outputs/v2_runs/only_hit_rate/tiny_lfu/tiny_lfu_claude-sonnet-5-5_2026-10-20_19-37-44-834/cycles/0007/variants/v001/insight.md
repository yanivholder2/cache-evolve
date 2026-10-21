# Variant v001 Insight

## Helped

- hit-rate: Raising `SKETCH_WIDTH_FACTOR` from 4 to 16 gave each count-min row four times as many counters. That lowers the chance that unrelated keys share a counter and inflate a candidate's or victim's estimated frequency. The admission duel therefore makes slightly better decisions. The gain is small but moves in the predicted direction.

## Hurt

- No reported dimension regressed. The wider sketch costs four times the metadata (`sketch_heap_bytes` is depth*width*4), but no metadata-size dimension is in the fitness vector, so that cost is unmeasured here. I could not read the overlay source in this session, so this comparison rests on the hypothesis text and the fitness vectors.

## Takeaway

On workload-a at size 2048 the sketch is slightly collision-limited, but only slightly. A 4x increase in width bought a small hit-rate gain. Together with the earlier finding that narrower rows lose, this suggests width is near diminishing returns. Larger improvements probably need changes to the admission rule, aging or reset period, or window sizing, not more counters.
