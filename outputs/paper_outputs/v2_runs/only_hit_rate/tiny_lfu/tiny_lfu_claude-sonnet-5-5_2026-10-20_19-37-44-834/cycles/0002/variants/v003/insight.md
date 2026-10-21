# Variant v003 Insight

## Helped

- (none) No measured dimension improved. I could not read the overlay source in this session, so this is judged from the hypothesis and the fitness vectors only.

## Hurt

- hit-rate: Hit-rate on workload-a at size 2048 fell below the baseline. The doorkeeper makes the first sighting of every key invisible to the counters, and `estimate` only adds 1 back. In a full cache, a newcomer competing against the LRU victim therefore tends to look colder than it is, and admission rejects more keys that would have hit.
- metadata-size, per-op-time: These dimensions are missing from the v003 fitness vector, so the change can't be assessed on them. The doorkeeper adds a bit array and extra hashing on every increment, so both are expected to cost something, but that was not measured.

## Takeaway

A doorkeeper filter is not a free win on top of this baseline sketch. On this LIRS trace the one-hit-wonder filtering did not offset the lost first-hit counts and the extra probes. Any gain from a doorkeeper probably depends on sketch pollution being a real problem for the workload, and on tuning the admission comparison to the doorkeeper's +1 offset, which this variant left unchanged.
