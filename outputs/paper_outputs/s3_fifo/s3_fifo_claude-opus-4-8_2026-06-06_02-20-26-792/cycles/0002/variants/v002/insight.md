# Variant v002 Insight

## Helped

- hit-rate: Raising `MAX_FREQ` from 3 to 5 lets hot main-queue entries accumulate a higher frequency count, so they survive more `evict_main` decrement-and-rotate passes before hitting `freq == 0`, retaining genuinely hot residents slightly longer in the already-enlarged main queue.

## Hurt

## Takeaway

Deepening the second-chance ceiling is a real but marginal lever on this workload: the gain over the parent is tiny, suggesting that with v002's ~10/90 partition most hot keys already clear the eviction gate well before they would have been capped at 3. The frequency ceiling and the partition ratio target the same retention behavior, so once main is large the extra headroom buys little additional protection.
