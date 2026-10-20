# Variant v003 Insight

## Helped

- per-op-time: The put tail (p99) improved slightly. With `MAX_FREQ` at 2, a main entry needs fewer decrement passes in `evict_main` before it is evicted, so the worst-case rotation is shorter. The gain is small, and the median latencies did not improve.

## Hurt

- hit-rate: Slightly lower. Capping the frequency counter at 2 means hot entries in main cannot build up a reserve of reuse credit. Some entries that would have survived another rotation at a cap of 3 were evicted early. The "cold entries age out faster" premise cost more in lost useful residents than it gained in freed space.
- per-op-time: The `get` median moved slightly the wrong way. This is probably noise, since `get` only increments a saturating counter and the cap does not change its work.

## Takeaway

Lowering `MAX_FREQ` while keeping the strict small-to-main admission test made main residency less sticky. Hit rate dropped, and the only benefit was a marginally shorter eviction tail. On this workload the frequency cap of 3 was doing useful work protecting reused main entries. A tighter cap is not a free way to trade residency for latency.
