# Variant v003 Insight

## Helped

- (none) hit-rate did not move relative to the baseline, so demoting displaced victims to the cold end gave no measurable gain on workload-a at this cache size.

## Hurt

- (none observed) The fitness vector only reports hit-rate, so metadata-size and per-op-time were not measured for this variant and cannot be judged. I also could not open the overlay source in this session, so I could not check the diff against the baseline.

## Takeaway

Changing where displaced items land in the stash/probing bucket (cold end instead of the baseline position) left hit-rate identical to the baseline on this trace. Either the demotion path rarely evicts a useful entry on this workload, or the change did not alter which entries get evicted. The cascade-eviction theory behind H03 is not supported by this result. The missing metadata-size and per-op-time dimensions mean the run cannot show whether the change had any cost.
