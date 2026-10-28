# Variant v003 Insight

## Helped

- hit-rate: did not move, which is what this variant had to show. The new `promotion_get` checks target → probing → stash instead of stash first. The uniqueness invariant (ADR 0004) means a key lives in at most one of those slots, so the same branch fires for every lookup. Hits, misses and item movement match the baseline exactly, on every trace and cache size.

## Hurt

_None._ Note: v003's fitness vector reports only `hit-rate`. `per-op-time`, the dimension H03 was meant to move, and `metadata-size` are missing from this evaluation, so the speed claim was not tested.

## Takeaway

Changing the lookup order is safe for hit-rate: putting the normal tier first and computing `stash_idx` only when needed leaves hits and item movement exactly as before. That makes it a free change to stack on later variants. The open question is whether skipping the up-to-64-item stash scan on target/probing hits actually lowers get p50. This run did not measure `per-op-time`, so v003 cannot answer that yet. It should be re-run with the latency profile before anyone counts it as a cost win.
