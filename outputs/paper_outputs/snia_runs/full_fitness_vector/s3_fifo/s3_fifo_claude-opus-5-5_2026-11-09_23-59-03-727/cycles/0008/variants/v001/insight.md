# Variant v001 Insight

## Helped

- hit-rate: Only web_0 at 16384 moved up, and only by a tiny amount. A few keys there are reused often enough to build up more than 7 hits while in main, so they now get a few more second-chance sweeps before eviction. The gain is too small to show the lever has real headroom.

## Hurt

- hit-rate: hm_0 at 16384 went down. That is the exact cell the hypothesis expected to improve. With a cap of 15, a key that was hot once can keep re-inserting itself through up to 15 `evict_main()` decrements after it goes cold. In a small, fast-turning main queue, that tenure blocks newer reused keys, so the stickiness that helped going from 3 to 7 now overshoots. Both traces at 131072 were exactly flat. That fits the parent's finding that a large main queue already keeps its hot set, so a cap above 3 does nothing there. The hit-rate summary went down slightly overall.
- per-op-time: `put` p99 rose the most of any timing stat, which is the risk the hypothesis flagged. A main victim can now cost up to 15 decrement-and-reinsert passes before `evict_main()` pops it, so the worst-case loop on a miss is about twice as long. `get` and `put` p50 drifted up by about the same small amount. `get()` still runs the same saturating add and `min`, so that part is likely noise.

(metadata-size did not move. `freq` is still a `u8`, so `Entry<u64,u64>` stays 24 bytes and `entry_fits_in_padding` still passes.)

## Takeaway

The main-queue frequency cap is saturated at 7. Going to 15 reverses the hm_0/16384 gain that cycle 5 got from 3→7, adds nothing at 131072, and makes the `put` tail slightly worse. This is the "flat or worse" outcome the hypothesis named as proof the lever is used up. Strongly reused hm_0 keys in a small cache need a different mechanism, such as a size-aware cap or a different small/main split, not more decrement credits that cold-but-formerly-hot keys can also keep using.
