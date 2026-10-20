# Variant v002 Insight

## Helped

(none)

## Hurt

- hit-rate: Doubling the ghost bound to `2 * main_size` did not surface additional re-promotable keys; on this Zipfian workload the cold-tail entries evicted from small are not re-requested even within `main_size` accesses, so the wider window retains only useless tombstones and the hit-rate fell below the parent.
- per-op-time: The larger ghost deque accumulates more stale lazy-deleted tombstones before the capacity guard fires, causing the drain loop in `evict_small()` to iterate longer on each put that evicts to ghost — visible as doubled p50 and p99 on the put path and a doubled p99 on get.

## Takeaway

Widening the ghost queue to `2 * main_size` costs latency (the lazy-deletion drain loop now wades through twice as many stale entries) without recovering any hit-rate, which means the workload's cold tail has no meaningful temporal locality beyond the original `main_size` window. A larger ghost is only worth its overhead when evicted keys actually return within the extended window; on this trace they do not, making ghost capacity expansion a net negative on both dimensions it touched.
