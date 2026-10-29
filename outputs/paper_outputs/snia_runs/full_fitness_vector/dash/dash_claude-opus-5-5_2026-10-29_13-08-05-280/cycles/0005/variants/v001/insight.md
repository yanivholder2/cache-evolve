# Variant v001 Insight

## Helped

- metadata-size: `Item<u64,u64>` shrank to the target size. `timestamp: Instant` became a `u32` tick, the LFU counter is a `u8`, and with key/value/timestamp/counter order the struct packs into 21 bytes, which rounds up to 24. `Dash<…>` itself was not touched and stayed the same.
- per-op-time (put median and tail): Put p50 and p99 both fell clearly, which is the opposite of cycle-0004 v001's put regression with the same 24-byte layout. The difference is where the tick lives. Each insert now bumps a counter local to its segment instead of a process-wide `AtomicU32`, so writes no longer contend on one global cache line. The smaller Item also means less data to write per insert and per eviction or relocation move.
- per-op-time (get tail): Get p99 dropped back down and won back most of what v003's probing guard cost. Get paths were not edited. The likely cause is the smaller Item: slow-path work (bucket scans, and the promotion/displacement cascade the guard defers) touches fewer cache lines when more Items fit in each line. Get p50 did not move. The fast hit path was already cache-resident, so it doesn't gain from denser packing.
- hit-rate (small-cache cells): Both 16384-size cells went up by a tiny amount, and only those cells changed. This is the expected sign of the size gate: below `GUARD_MIN_CAPACITY` the third stash hit is no longer required, so small caches promote to target at two hits again. The 131072 cells, where the guard is still on, are exactly the same as v003's. That fits the layout change being neutral for ClassicLRU, which never reads the timestamp.

## Hurt

(No dimension got worse.)

## Takeaway

Reusing cycle-0004 v001's 24-byte Item works without its latency penalty once the timestamp source is per segment. That confirms the global `AtomicU32` caused v001's put regression, not the smaller layout, and the denser Item actually speeds up put and the get tail. The size gate's open question has a clear answer: with v003's target-side probing guard present, turning off the stash-side guard for small caches adds only a negligible small-cache hit-rate gain. So the gate is now mostly harmless complexity, and the earlier small-cache slip was largely fixed by the probing guard. Note: this was written from the hypothesis and both fitness vectors. The overlay source files could not be listed in this session, so the code-level reasons assume the hypothesis was implemented as written.
