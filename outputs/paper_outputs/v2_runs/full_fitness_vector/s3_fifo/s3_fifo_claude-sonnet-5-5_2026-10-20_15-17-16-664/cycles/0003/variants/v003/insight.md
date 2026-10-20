# Variant v003 Insight

## Helped

- (none) No dimension improved. Metadata-size is unchanged because the edit adds only a local counter in `evict_main` and no struct fields.

## Hurt

- hit-rate: The rotation cap makes `evict_main` drop the tail even when its freq is still above zero. This sometimes evicts an entry that the uncapped second-chance loop would have kept, so hit rate regressed slightly on workload-a.
- per-op-time: Get and put latencies both got worse. `get` is untouched by the diff, so the get regression points to measurement noise or machine variance, not the cap. The put-side regression also can't be credited to the cap, which only shortens the loop. The hypothesis that unbounded rotation was the latency tail was not supported.

## Takeaway

With `MAX_FREQ` = 3, the rotation loop is already effectively bounded. Each pop decrements a freq that saturates at 3, so a full-queue rotation is rare, and capping at `MAX_FREQ + 1` saves little work while occasionally evicting a still-hot entry. The cap trades hit rate for no measurable speed. The latency measurements are noisy enough (the `get` path moved without any code change) that they can't validate micro-optimizations of this size.
