# Variant v003 Insight

## Helped

- hit-rate: Gating the stash→target promotion when the target is full skips the read-path `promote_to_target` cascade that would evict a target LRU resident, so fewer already-resident items get pushed out — retention rose instead of falling, the reverse of the hypothesis's predicted residency loss from under-promotion.

## Hurt

- per-op-time: Adding the `is_full()` check and the in-stash-touch branch on every stash-hit get raised both get-p50 and get-p99 rather than lowering the tail — the full-target cascade this gate removes was rare enough on workload-a that the per-hit branch cost outweighs the tail work it eliminated, inverting the intended get-tail win.

## Takeaway

The cascade gate landed exactly inverted from the hypothesis: the work it removed was a hit-rate *gain* (skipping eviction of a target resident retains more useful keys), not the predicted residency cost, while get latency *regressed* because the cascade it was meant to cut almost never fires here, leaving only the added per-get branch. Gating read-path promotion on this workload buys hit-rate at the price of get speed — the opposite of the get-tail-for-hit-rate trade it was designed to make.
