# Variant v002 Insight

## Helped

(No dimension improved. metadata-size did not move: the change only alters a value written to the existing `u8` `freq` field, and `Entry<u64,u64>` stays 24 bytes.)

## Hurt

- hit-rate: All four trace/size points dropped, hm_0 at 16384 included, which was the target. Setting `entry.freq = 1` in the `evict_small()` graduation branch means every graduated key now takes up main for one extra full rotation, even if it is never hit in main. That extra residency comes out of the space for keys that are actually reused inside main. It also pushes eviction pressure onto ghost-admitted keys, which still enter main at freq 0 through `Entry::new` and so are now the easiest victims, even though a ghost hit is a proven re-reference.
- per-op-time: `put` p99 rose more than the run-to-run drift seen in the parent cycle. This fits the mechanism. Every graduated key now forces at least one extra `pop_back`/decrement/`push_front` step before `evict_main()` finds a freq-0 victim, so the sweep on a miss gets longer. The `get` p50 change is about 1 ns, and since `get()` is unchanged it is most likely noise.

## Takeaway

On this policy, any freq carried across graduation hurts, even a single unearned credit. Cycle 3's full carry-over finding extends down to the minimum: the reset to 0 in the parent was already the right setting. Two hits in small show recency inside probation, not that the key will be reused in main. Main is better at keeping keys when graduated keys have to earn their second chance with a hit in main, so the graduation-credit lever is exhausted. More work on hm_0 at 16384 should look somewhere other than freq granted at graduation.
