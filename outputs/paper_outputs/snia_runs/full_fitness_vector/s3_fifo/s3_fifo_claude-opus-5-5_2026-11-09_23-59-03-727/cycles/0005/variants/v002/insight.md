# Variant v002 Insight

## Helped

- hit-rate: Raising `MAX_FREQ` from 3 to 7 lets a key hit many times inside main survive more `evict_main()` second-chance sweeps. The only clear gain is on hm_0 at the small cache size (16384). There, main is short and turns over fast, so a cap of 3 was evicting keys that were still hot. web_0 at 16384 rose by a smaller amount, and the summary went up slightly. As the hypothesis predicted, the graduation reset in `evict_small` keeps the higher cap out of small-queue admission.
- per-op-time: `get` p50/p99 and `put` p50 dropped slightly, and the summary moved by about 1 ns. The change is only a constant swap, and `get()` runs the same saturating add and `min` as before, so this is almost certainly run-to-run noise rather than an effect of the change.

## Hurt

- hit-rate: At the large size (131072), hm_0 and web_0 were flat to very slightly down. When main is big enough to hold the hot set across sweeps, extra stickiness gives no benefit and can keep formerly-hot keys a little longer than they deserve.
- per-op-time: `put` p99 rose slightly. This fits the risk the hypothesis named: a main victim can now be reinserted up to 7 times instead of 3 before it is evicted, so the worst-case `evict_main()` loop on a miss is longer. The change is small enough that it could also be noise.

(metadata-size did not move: `freq` is still a `u8`, and `Entry<u64,u64>` stays 24 bytes, as `entry_fits_in_padding` asserts.)

## Takeaway

On the two-filter parent, a deeper main-queue frequency cap pays off only where main is too small to keep its hot set. That means hm_0 at 16384. At 131072, three second chances were already enough, so cap depth is not the main untapped lever for retaining keys in main. Future main-retention changes should target small-cache behavior on hm_0, for example a size-aware cap or a different split between the small and main queues. Raising the cap further is unlikely to help, because it adds some `put`-tail cost for little gain at large sizes.
