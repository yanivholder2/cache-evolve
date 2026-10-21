# Variant v001 Insight

## Helped

- (none measured): The hypothesis targeted metadata-size by halving the sketch's counters per row. The fitness vector has only a hit-rate dimension, so any memory saving is not visible in the evaluation.

## Hurt

- hit-rate: Halving the row width makes more keys share each counter. Conservative update over-estimates frequencies more often, which inflates the estimates in the tie-admit duel (`>=` for repeat keys, strict `>` for one-hit keys). That lets cold keys beat or tie warm victims, and the hit-rate falls slightly below the parent's. The drop is larger than the 0.002 tolerance the hypothesis set.

## Takeaway

The u8 sketch at this width is not as over-provisioned as cycle 1 suggested. Even with conservative update and independent row seeds, halving the width costs more hit-rate than the hypothesis allowed. Because the evaluation has no metadata-size dimension, the trade cannot be shown as a win. Width reductions need a size metric in the fitness vector, or a gentler step than 2x, to be judged.
