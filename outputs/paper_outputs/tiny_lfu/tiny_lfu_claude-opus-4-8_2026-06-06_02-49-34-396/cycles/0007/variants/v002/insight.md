# Variant v002 Insight

## Helped

- per-op-time: Dropping `SAMPLE_VICTIMS` from 5 to 3 shortens the eviction forward-walk and cuts two `sketch.estimate` calls per eviction, nudging the put p99 tail — the cross-cycle soft spot the hypothesis targeted — down slightly.

## Hurt

- per-op-time: The summary regressed because get p50/p99 and put p50 all rose, but the smaller victim sample only touches the put eviction walk and cannot affect the get path at all, so this swing reflects benchmark run-to-run variance rather than the change.

## Takeaway

Trimming the victim sample below 5 confirms the premise: hit-rate stayed essentially flat (one-tenth of a basis point), so eviction quality on this trace is bound by the sketch estimate, not sample depth. The only real, attributable effect is a modest shave of the put p99 tail; the apparent get/put-p50 regression is noise on paths the edit never touches, and metadata-size is untouched since this is a pure constant change.
