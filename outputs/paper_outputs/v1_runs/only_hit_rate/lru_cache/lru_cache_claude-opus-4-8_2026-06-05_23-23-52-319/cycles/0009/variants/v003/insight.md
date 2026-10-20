# Variant v003 Insight

## Helped

- hit-rate: Routing new keys through the FIFO `A1in` admission queue and only promoting a key to the main `Am` LRU on an `A1out` ghost hit keeps one-hit-wonders out of the protected main list, so reused keys survive churn and the workload-a hit-rate rose over the baseline single-LRU.

## Hurt

- (none measured): the only evaluated dimension moved in the favorable direction; no regression is visible in the fitness vector.

## Takeaway

On this trace, a 2Q structure — FIFO admission with ghost-driven promotion rather than in-window recency — is the structural axis that finally beats the single-LRU baseline: filtering scan/one-hit traffic before it can evict proven-reuse entries is what lifts the hit-rate.
