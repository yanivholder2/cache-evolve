# Variant v003 Insight

## Helped

- hit-rate: A key now has to be hit twice in stash before it moves to its target bucket. Keys reused only once stay in the larger stash (24 buckets vs 8) and no longer evict the LRU resident of a full target bucket. That cuts the churn in the scarce normal tier that the baseline's promote-on-first-hit path caused on the LIRS workload. Items demoted out of the normal tier get one stash hit pre-credited, so they can come back after a single hit. This keeps the filter from penalising keys that had already earned residency.

## Hurt

## Takeaway

The stash→target edge is the place to filter what gets into the normal tier. Requiring a second stash hit works on this trace because the stash has room to hold unproven keys while they wait. Resetting the reused `lfu_counter` on promotion and pre-crediting it on demotion makes the filter apply mainly to new keys rather than to returning residents. v003 was evaluated only on hit-rate, so the hypothesis's claim of lower get p50 (from skipping remove+insert+cascade on first hits) is untested. The same goes for the claim that `Item` size is unchanged.
