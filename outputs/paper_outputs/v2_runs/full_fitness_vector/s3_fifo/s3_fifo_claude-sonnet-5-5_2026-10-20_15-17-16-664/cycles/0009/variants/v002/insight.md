# Variant v002 Insight

## Helped

- (none): no dimension improved. The adaptive small-queue sizing did not change which entries were admitted or evicted on the evaluated workload.

## Hurt

- metadata-size: `S3Fifo` gained eight bookkeeping fields (`min_small`, `max_small`, `step`, `window_puts`, `ghost_hits`, `small_evict_to_ghost`, `ghost_bound`, and a now-mutable `small_size`). The struct grew substantially, while `Entry` stayed the same size.
- per-op-time: `put` now increments `window_puts` and checks the window boundary on every miss. It also adds a ghost-hit counter and a second `while` drain loop over small and main. Both `get` and `put` latencies rose, with the put tail worst. The `get` path is unchanged, so part of the `get` slowdown is probably layout or noise rather than the algorithm.

## Takeaway

The ARC-style feedback loop was inert on this trace. The ghost-hit ratio most likely stayed inside the 0.1–0.5 dead band, or the 1–10% bounds let `small_size` barely move. The hit rate therefore matched the fixed-3% parent exactly, and the counters and window check only added cost. Adaptive sizing needs evidence that the ratio actually leaves the dead band, for example by logging `small_size` over time, before it is worth its per-put overhead and extra state.
