# Variant v002 Insight

## Helped

- hit-rate: Held level with the parent. Skipping the promotion swap on put-of-existing-key did not lose recency signal, since get-hit transpose promotion still carries it.
- metadata-size: Unchanged. The change only altered control flow in the put path and added no fields to `Item` or the cache struct.

## Hurt

- per-op-time: The put p50 did not improve, so the expected saving from dropping the put-hit reorder did not appear. The put tail got worse, which suggests the early-return/single-scan restructure added variance rather than removing work. The get tail got tighter, but that is more likely noise than an effect of this change, because the get path was untouched. The summary is driven by get and did not move.

## Takeaway

Removing the reorder on put-hits is neutral for hit rate but buys no median put latency here. The reorder was not a meaningful cost, probably because the workload's puts are mostly misses, or because the bucket scan is already cheap. Further put-latency gains would have to come from the miss/insertion path, not from put-hit handling.
