# Variant v002 Insight

## Helped

- hit-rate: No measurable gain. The hypothesis changed protected-overflow demotion to pick the lower-estimate of the two LRU protected entries, but hit-rate on workload-a at size 2048 is essentially identical to the parent's. A difference in the fourth decimal place is noise-level and slightly negative.

## Hurt

- hit-rate: Marginally lower than the parent, but within noise. If the change had any effect, it did not help. I could not read the overlay source in this session, so this rests on the hypothesis text and the fitness vectors.

## Takeaway

Using the sketch estimate to choose between the two oldest protected entries did not change which entries get evicted in any way that matters on workload-a. Either the two LRU candidates usually have similar frequency, or demotion only moves an entry to probation MRU, where it stays resident until probation evicts it. Two successive cycles of tuning segment internals (the protected size in the parent, the demotion choice here) have left hit-rate flat. The next lever is therefore more likely the admission path, the sketch (aging, reset, width) or the window size.
