# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Raising `MAX_FREQ` from 3 to 7 left hit-rate on the lirs workload-a trace at size 2048 exactly where the parent put it, so the premise that bursty hot keys in main are being evicted too early under the cap of 3 did not show up at this size. The change is a single constant, and `evict_main()` only sees a difference when a main resident's frequency exceeds 3, so the null result suggests few main residents on this trace accumulate more than three hits before being rotated. Only hit-rate is reported in the fitness vector, so the metadata-size and per-op-time effects the hypothesis asked for are unmeasured here.
