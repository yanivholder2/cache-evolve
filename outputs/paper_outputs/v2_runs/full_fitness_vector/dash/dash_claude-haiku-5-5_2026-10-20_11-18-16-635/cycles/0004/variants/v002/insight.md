# Variant v002 Insight

## Helped

- hit-rate: The saturating access counter biases `evict_one()` away from recently hit slots, so bucket residents with repeated target hits survive eviction longer on the LIRS workload. This is the mechanism H02 predicted, and it is the only dimension where the variant clearly beat its parent.
- per-op-time (get): The get path was left as in cycle 0001 v002 (stash scan skipped on common target and probing hits), and the counter only adds work on the eviction branch, so get latency held at the parent's level.

## Hurt

- per-op-time (put): `evict_one()` now has to find the lowest-counter slot instead of taking the parent's existing victim, so every put that lands in a full bucket pays for the extra comparison and tie-break scan. Put is the operation that runs eviction, which is why it moved while get did not.

## Takeaway

Frequency-aware victim selection buys hit-rate on this workload without touching insert position or the get path, and the cost lands on the put/eviction path rather than on lookups. The hit-rate gain is measured on workload-a only, so it should not be treated as a general result until a second trace confirms it.
