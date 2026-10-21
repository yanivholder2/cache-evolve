# Variant v002 Insight

## Helped

- hit-rate: Halving `small_target` (`capacity / 40`, 102 → 51 at size 2048) and growing `ghost_cap` to `3 * capacity / 20` (256 → 307) keeps the combined `small + ghost` admission window at 358. The 51 resident slots that leave probation go to `main`, where reused keys keep getting lazy reinsertion. A key evicted from `small` with `freq == 0` now stays in the ghost longer and is still admitted straight into `main` if it returns within roughly the same window as before. So `main` gets more room and the cache loses little of its ability to spot reuse. On workload-a that was a net gain. `get`, `put`, `evict_one`, `push_ghost` and `MAX_FREQ` are unchanged, so the split is the only thing that changed.

## Hurt

(No measured dimension regressed. The cost shows up inside hit-rate rather than as a regression: the gain was marginal. Two things push against the extra `main` space. First, a key now has only about 51 small-queue evictions to be re-referenced before it leaves `small`, so fewer keys earn promotion with a hit while in `small`. Second, more keys reach `main` through a ghost hit instead, and they arrive with `freq = 0`, which gives them no first-pass protection in `main`'s reinsertion loop.)

## Takeaway

On workload-a at size 2048, the "keep `small + ghost` constant and shift slots into the ghost" rule from cycle 6 still holds one step further. A ghost entry is cheaper probation than a resident `small` slot. But the gain from this step was small, so it is not clear that another halving of `small` would still help. The likely limit is the one the hypothesis named: keys entering `main` via the ghost with `freq = 0`, and a reuse window in `small` that keeps shrinking. A cheap next test is to make the same move at a second cache size. Another is to give ghost-admitted keys a starting `freq` of 1. That would tell whether more of `small` can safely move to the ghost.
