# Variant v003 Insight

## Helped

## Hurt

## Takeaway

The promotion gate is built as specified. Once protected is full, a probation hit enters protected only if its sketch estimate is at least the minimum over the `DEMOTE_SAMPLE = 4` oldest protected entries. Otherwise it gets a second chance at probation MRU. Workload-a's hit-rate did not move beyond noise, so the gate almost never changes an outcome. It compares against the *coldest of the four LRU-oldest* protected entries, and those are the protected entries with the weakest frequency evidence. Meanwhile the probation key being hit has already beaten a sampled victim to get in, and then collected another increment. With non-aging lifetime counts, the hit key nearly always ties or exceeds that floor. When the gate does block a hit, the blocked key moves to probation MRU, which already keeps it outside the 8-entry victim sample for a while. So blocking and promoting give it about the same protection. Protected-entry selection has now been tried on recency, demotion target and promotion gating, and none moved hit-rate. On this trace, protected membership is not the binding constraint. Future levers should change which keys win the admission contest or how counts age, not how entries move between SLRU segments.
