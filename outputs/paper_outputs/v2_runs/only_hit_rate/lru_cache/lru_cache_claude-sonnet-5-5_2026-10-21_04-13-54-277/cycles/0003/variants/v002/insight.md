# Variant v002 Insight

Note: I could not list or read the overlay source in this session. Only the hypothesis and the two fitness vectors were available. The analysis below rests on those.

## Helped

- (none): hit-rate is identical to the parent's, so the ghost list gave no measurable benefit.

## Hurt

- (none): hit-rate did not regress. The parent's score was matched exactly, unlike cycle 0002's ghost variant, which lost hit-rate.

## Takeaway

Re-admitting ghost hits at probation MRU, instead of straight into protected, removed the regression the earlier ghost variant caused. It did not produce any gain on workload-a at size 2048. An identical hit-rate to the parent suggests the ghost path rarely changed an admission or eviction decision on this trace. It is also possible the change was not exercised at all. If so, evicted-then-reinserted keys are too rare, or too far apart in time, for a capacity-sized ghost list to matter. The extra bookkeeping is unjustified unless a different trace shows a benefit.
