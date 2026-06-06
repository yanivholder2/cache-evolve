# Variant v003 Insight

## Helped

- hit-rate: Capping the ghost queue at `main_size / 2` (~921 keys for a 2048-slot cache, down from ~1843 in v001) pruned the oldest, coldest tracked evictions, concentrating ghost membership on the items most recently expelled from main — where re-use probability is highest — so ghost hits led to more genuinely productive promotions.

## Hurt

## Takeaway

With v001's enlarged main queue, items linger longer before eviction, meaning keys that finally age out tend to be colder than in the baseline; tracking all of them in a full-`main_size` ghost window fills it with entries that are unlikely to be re-requested. Halving the ghost capacity discards that cold tail and keeps only the freshest eviction signal, which is sufficient and less noisy. The improvement in hit rate confirms that for this workload, ghost *precision* — tracking only the most recently displaced keys — is more valuable than ghost *breadth*.
