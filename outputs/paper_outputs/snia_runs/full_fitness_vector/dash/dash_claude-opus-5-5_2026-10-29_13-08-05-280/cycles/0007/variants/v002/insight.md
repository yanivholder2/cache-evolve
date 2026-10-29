# Variant v002 Insight

## Helped

- hit-rate (large-cache cells): Both 131072 cells held or went up slightly, and the 16384 cells did not change. The refutation condition did not trigger. When the guarded promotion targets a full probing bucket, dropping that bucket's LRU entry outright is no worse than the parent's cascade, which re-homed it through neighbour retry, target-LRU eviction and stash insert. The cascade's re-homing was not keeping useful keys. Leaving the target slot free may also help a little, because later puts into that bucket no longer have to evict a resident.
- metadata-size: No change, as intended. The change only alters control flow in one arm and adds no fields to Item or the segment.

## Hurt

- per-op-time (get and put, p50 and p99): All four latency figures got much worse. But the change cannot explain most of that. It only touches the rare `is_hottest` + full-probing + guard-passed arm, and in that arm it does strictly less work than the cascade. Yet put p50 and get p50 regressed as much as the tails, and put is barely affected by this change. A drop that uniform, including on paths the change never touches, points to a measurement artifact, such as host contention or a different bench configuration or build profile in this run. It could also be an unintended global side effect in the implementation, but the change itself is not a plausible cause. Either way, the hypothesis's tail-latency confirmation was not met.

## Takeaway

On hit-rate, the change worked: the probing evictee can be discarded instead of cascaded with no loss at either cache size. That makes the cheap "evict and leave a free slot" path a safe base for later tail-latency work. The latency result can't be used to judge the hypothesis, because the regression is uniform across ops and percentiles while the change only touches one rare get-path arm. Re-benchmark this variant next to its parent on the same host before deciding whether removing the cascade cuts get/put p99. Note: the overlay source and eval/ contents could not be listed in this session, so the code-level reasons assume H02 was implemented as written.
