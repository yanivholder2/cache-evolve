# Variant v002 Insight

## Helped

- None measured. hit-rate is the only reported dimension, and it did not improve over the parent.

## Hurt

- hit-rate: Power-of-two-choices placement balanced bucket occupancy, but it did not turn into more hits. It came out marginally below the parent on workload-a, which is within plausible single-trace noise. Lookup now probes two buckets, so any extra cost is not captured by the reported dimensions. I could not read the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

Spreading admitted keys across the less-loaded of two candidate buckets did not raise hit-rate. Forced evictions from hot buckets were therefore not what limited hit-rate here. Eviction quality (which entry is dropped) and admission matter more than bucket load balance, so bucket-structure changes look saturated for this workload, and the added second probe and locking complexity is not justified.
