# Variant v001 Insight

## Helped

- hit-rate: The stash-hit branch of `DashSegment::get` keeps the parent's MRU landing (`InsertPosition::Back`) and now adds a second-hit gate, so a re-referenced stash item is only promoted on its later hit. On the LIRS workload-a trace this is a small gain over the parent, consistent with the stacked change keeping items that get hit twice resident while leaving one-off stash hits in place.

## Hurt

## Takeaway

Stacking the second-hit promotion gate on top of the MRU landing produced a small hit-rate gain over the parent on the one trace measured, which supports the idea that the two levers are complementary rather than redundant. Because the only evaluated trace is LIRS workload-a at size 2048, and the variant does not touch metadata size or per-op time, the gain is not yet evidence it generalizes across workloads or capacities.
