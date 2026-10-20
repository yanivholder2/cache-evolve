# Variant v002 Insight

## Helped

- (none): the hypothesis targeted worst-case per-eviction work, and hit-rate is the only dimension measured, so no measured dimension improved.

## Hurt

- hit-rate: a small regression against the parent. Forcing eviction of the main-queue tail after 8 reinsertions sometimes evicts an entry with freq>0 that the unbounded second-chance scan would have kept.

## Takeaway

The cap on reinsertions per main eviction was expected to be neutral, but it cost a little hit-rate on workload-a. That suggests the long reinsertion scans do real work here by protecting hot entries, so the unbounded scan is not just a worst-case cost. The latency benefit the cap was meant to buy cannot be judged, because no timing dimension exists in the fitness vector. I could not read the overlay sources in this session because file-listing and shell tools were unavailable, so this interpretation rests on the hypothesis text and the fitness vectors. The result comes from a single trace and cache size, so the regression may be small enough to be noise.
