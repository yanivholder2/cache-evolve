# Variant v001 Insight

## Helped

## Hurt

## Takeaway

H01 folds resident-key membership into the per-slot queue-state bit so `contains_resident()` no longer probes a side table, with the stated aim of lowering metadata-size and per-op-time while leaving eviction order and hit-rate alone. Hit-rate is flat against the parent, which is consistent with the claim that eviction and promotion were left untouched. The fitness vector reports only hit-rate, so the metadata-size and per-op-time effects the hypothesis targets are unmeasured here, and the overlay source was not inspected in this run, so the bit-folding itself is unconfirmed.
