# Variant v003 Insight

## Helped

- per-op-time (get): moving the stash scan (up to 64 entries) to the end of `promotion_get` and computing `stash_idx` lazily means hits in the target or probing bucket no longer pay for the stash scan or the extra hash. Get p50 and p99 both fell to well under half of the baseline, so on workload-a most hits must be served from normal buckets, not from the stash.
- per-op-time (put): put latency also fell a lot, even though no put code changed in this diff. Either the put path does a lookup through `promotion_get`, or this is run-to-run noise. Re-measure it before crediting the change.

## Hurt

## Takeaway

The stash-first lookup order was the main cost on the baseline's get path. The uniqueness invariant makes the order of the checks irrelevant to the result, so checking target → probing → stash removed most of that cost without changing hit-rate or metadata-size. This should be the default lookup order for later variants. It also tells us hits on this workload come mostly from normal buckets, so stash-side optimisations will pay off less than work on the target/probing tier.
