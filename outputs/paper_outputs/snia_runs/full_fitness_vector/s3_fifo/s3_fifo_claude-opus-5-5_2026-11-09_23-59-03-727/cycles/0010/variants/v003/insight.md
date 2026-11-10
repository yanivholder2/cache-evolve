# Variant v003 Insight

## Helped

(No dimension clearly improved. `get` p99 is unchanged, and the ~1 ns moves in `get`/`put` p50 match the run-to-run noise noted in the parent's insight. metadata-size did not move: `MAX_SWEEP_STEPS` is a `const` and `steps` is a stack local, so `Entry<u64,u64>` and `S3Fifo<u64,u64>` keep the same layout.)

## Hurt

- per-op-time: `put` p99 rose by much more than noise, the opposite of what H03 predicted. Capping `evict_main` at 32 reinsertions did not cut the tail. One likely cause is that the parent's tail never came mainly from long sweeps, since each sweep step is just a VecDeque pop/push on a `u32` plus a byte decrement. The other is the force-evict path. Every force-evicted entry has freq > 0, so it is always `reused` and always calls `push_ghost`. That adds ghost inserts, more frequent oldest-ghost pop loops, and more frequent O(2·main_size) `ghost.retain` compactions. Each of those is a far bigger single-op stall than a 32-step sweep. This is inferred from the code, not profiled.
- hit-rate: All four trace/size cells dropped slightly, at both 16384 and 131072. So the cap does bind on these traces: sometimes the 33rd back entry is still hot when it is evicted, and a colder freq-0 entry deeper in main survives. The reuse-gated ghost return path does not fully make up for that, because a returning key comes back with freq 0 and has lost the credit it had built up.

## Takeaway

Bounding the main-queue second-chance sweep is the wrong lever for `put` p99 in this policy. The cap binds often enough to cost a little hit rate everywhere, and forced evictions always produce ghost inserts. That moves work into `push_ghost`/`retain`, which has a heavier tail, so the "sweep length drives the `put` tail" claim from cycles 8–9 looks wrong or incomplete. Before trying another sweep bound, profile where the parent's `put` p99 is spent, especially the ghost compaction in `push_ghost`. Any future forced eviction should probably skip ghosting, or the ghost cost should be amortized first.
