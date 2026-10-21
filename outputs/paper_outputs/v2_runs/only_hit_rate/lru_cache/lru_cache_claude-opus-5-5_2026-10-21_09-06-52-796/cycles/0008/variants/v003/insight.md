# Variant v003 Insight

## Helped

- hit-rate: Halving `small_target` again (`capacity / 40` → `capacity / 80`, 51 → 25 at size 2048) and growing `ghost_cap` to `13 * capacity / 80` (307 → 332) keeps the `small + ghost` admission window essentially fixed (357 vs 358). It frees about 26 more resident slots for `main`, where reused keys keep their lazy reinsertion. Hit rate still went up, so on workload-a a ghost entry is still cheaper probation than a resident `small` slot, even at this ratio. `get`, `put`, `evict_one`, `push_ghost` and `MAX_FREQ` are byte-identical to v002, so the split is the only cause of the change.

## Hurt

(No measured dimension regressed. The gain was smaller than the already-marginal `/20` → `/40` step, which means the opposing costs nearly cancel the extra `main` space. With only about 25 small-queue evictions before a key leaves `small`, few keys get re-referenced in time to be promoted with `freq > 0`. More reuse is caught only by the ghost instead, and those keys enter `main` with `freq = 0`, so they get no protection on their first pass through `main`'s reinsertion loop.)

## Takeaway

The shift-`small`-into-ghost curve has now flattened to almost nothing on workload-a at size 2048. The `/80` step added only a sliver over `/40`, so another halving (`/160`, where `small` would drop to about 12) is unlikely to be worth a cycle. The `small`/ghost split can be treated as settled near `capacity / 40` to `capacity / 80`. Further gains should come from the bottleneck this variant made larger: ghost-admitted keys entering `main` with `freq = 0`. Giving them a starting credit, or carrying credit over as H01 proposes, is the natural next step, and the ratio should be re-checked at a second cache size before anyone relies on it.
