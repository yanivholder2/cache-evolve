# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Gating admission on a count-min frequency estimate against the LRU tail left workload-a hit-rate at size 2048 unchanged from baseline at the reported precision, so the gate either rarely rejected a candidate that would have been hit later or its rejections did not change which keys were resident. The eval artifacts contain no admission counts, so the hypothesis's check that the gate engaged cannot be confirmed from this variant's results. The metadata-size and per-op-time dimensions are also absent from this variant's fitness vector, leaving the sketch's memory and time cost unmeasured.
