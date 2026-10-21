# Variant v002 Insight

## Helped

(None measured. Only hit-rate was recorded, and it did not improve. per-op-time and metadata-size were not re-measured. The code adds no per-Item fields, so metadata per item should be unchanged. It does add a 16-entry `u64` ring and an index per bucket. Every ClassicLRU miss now runs one `DefaultHasher` fingerprint and a 16-entry scan, plus a second hash for the evicted key.)

## Hurt

- hit-rate: A key whose fingerprint is still in the 16-entry ghost ring now goes straight to the MRU tail, skipping the midpoint. On workload-a, these returning keys look like they mostly don't get a second hit before they reach the eviction end again. They take a full bucket lifetime of protected space and push resident keys toward the eviction end, and some of those keys would otherwise have hit. That undoes part of what v001's midpoint filter protected.

## Takeaway

A ghost list exactly one bucket deep (16 fingerprints for 16 slots) flags keys whose reuse distance is between one and two bucket lifetimes. On workload-a, a single recurrence in that window does not predict another one soon, so giving these keys MRU placement for free gives back the protection v001 earned by making new keys prove reuse with a resident hit. If ghost-based admission is tried again, it should be more cautious, not abandoned outright. For example, place ghost hits partway between the midpoint and the tail, or shrink the ghost ring so that only keys evicted very recently count. Otherwise the midpoint-only policy of v001 stays the better base to build on.
