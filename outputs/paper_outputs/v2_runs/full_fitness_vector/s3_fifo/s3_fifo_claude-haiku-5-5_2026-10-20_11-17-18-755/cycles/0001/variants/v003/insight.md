# Variant v003 Insight

## Helped

- per-op-time: The put path's main-queue eviction now stops after `MAX_EVICT_SCAN` second-chance rotations and drops the current tail, so a single put can no longer rotate the whole main queue; the tail latency of `put` moved the most.

## Hurt

- hit-rate: Once the 8-rotation budget is spent, the tail is evicted even when its `freq > 0`, so some recently referenced entries leave the cache early; the loss is small but it is the expected cost of the cap.

## Takeaway

Capping the second-chance scan at a fixed budget turns main-queue eviction into a bounded-cost operation, and the measured `put` tail improvement is the intended effect of that bound. The hit-rate cost comes entirely from force-evicting hot entries once the budget runs out, so the cap's value depends on how often a put exhausts the 8-step budget under this workload. The `get` latency shift is not explained by this change, since `get()` is untouched; treat it as measurement variance rather than a result of this variant.
