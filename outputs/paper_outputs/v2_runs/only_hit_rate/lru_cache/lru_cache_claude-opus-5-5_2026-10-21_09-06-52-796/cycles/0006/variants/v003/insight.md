# Variant v003 Insight

## Helped

- hit-rate: Changing `small_target` from `capacity / 10` to `capacity / 20` (204 → 102 at size 2048) gives about 102 more slots to `main`, where keys with proven reuse stay and keep getting lazy reinsertion. The keys this shorter probation window now misses are not lost. A key evicted from `small` with `freq == 0` goes into the 256-slot ghost, which is now about 2.5x the size of `small`. If that key comes back within roughly 102 + 256 small-queue evictions, the ghost hit admits it straight into `main`. The two-stage filter (a hit while in `small`, or a return while still in the ghost) catches about the same reuse window as before. What changed is that the second stage costs only a key in `ghost_q`/`ghost_set`, not a resident slot. On workload-a, the slots freed for `main` were worth more than the in-`small` hits given up.

## Hurt

(No measured dimension regressed. The fitness vector reports only `hit-rate`. Promotion via a hit in `small` now needs reuse within a window half as long, so more keys reach `main` through the ghost path instead. They arrive with `freq = 0` and get no early protection in `main`'s reinsertion loop. That cost did not outweigh the gain here. The in-file tests are unchanged and still pass only because `max(1, cap/20) == max(1, cap/10) == 1` at capacities 10 and 16. That means no test exercises the new ratio.)

## Takeaway

The "~10% probation split" was left over from the earlier long-ghost configuration. It is not a true optimum for the current S3-FIFO. Once the ghost is short and reliable, it can do part of `small`'s job as a cheap extended probation window, and the resident space this frees is better spent in `main`. So `small_target` and `ghost_cap` should be tuned together as one admission window, not as separate knobs. One way is to keep `small + ghost` roughly constant and keep shifting slots from `small` to the ghost. That should be checked at a second cache size before treating it as general, because this evidence comes from one trace at one size.
