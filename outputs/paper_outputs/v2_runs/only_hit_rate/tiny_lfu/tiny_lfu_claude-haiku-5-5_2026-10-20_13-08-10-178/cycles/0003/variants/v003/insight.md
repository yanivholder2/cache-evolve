# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Narrowing the sketch counters from `u16` to `u8` with saturation at 255 left hit-rate on workload-a at size 2048 identical to the parent, so the admission comparison did not visibly change on this trace. That means the saturation point is not yet a binding constraint at 255 for this workload, which narrows the open question from v001 about admission ties: they are not triggered by the hottest keys here even at the sharpest saturation level tested. The footprint gain the hypothesis targeted is real by construction but unmeasured, since the fitness vector only reports hit-rate.
