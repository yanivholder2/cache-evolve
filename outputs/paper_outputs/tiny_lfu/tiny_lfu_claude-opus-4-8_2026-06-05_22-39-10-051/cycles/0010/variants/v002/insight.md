# Variant v002 Insight

## Helped

- (none observed): the only measured dimension is hit-rate, and halving sketch depth did not raise it; the estimator-work savings the hypothesis chased are real but invisible to this fitness vector.

## Hurt

- hit-rate: dropping `SKETCH_DEPTH` from 4 to 2 shaved a sliver off hit-rate — with only two rows the count-min `min` reduction loses one of its over-estimate corrections, so a few frequency estimates run slightly high and admit/protect marginally worse keys.

## Takeaway

At `width = capacity * 64` depth-2 is *almost* but not quite redundant: the second-to-fourth rows were buying a last fraction of a percent of estimator accuracy, not nothing. The redundancy floor for this estimator sits at depth-4, not depth-2 — the free capacity proven at depth-8 is already fully spent by depth-4, so trimming further trades a measurable hit-rate dip for unmeasured hash savings.
