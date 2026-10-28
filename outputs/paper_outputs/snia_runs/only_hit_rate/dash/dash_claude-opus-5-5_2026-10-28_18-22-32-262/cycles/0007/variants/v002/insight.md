# Variant v002 Insight

## Helped

- hit-rate (128K, `hm_0`): This is the only cell that moved noticeably, and it moved up. A large `hm_0` cache seems to be where stash buckets overflow while the victim's own target or probing bucket still has a free slot. Rescuing that victim into the LRU end of the normal bucket keeps it resident instead of sending it to the ghost. If it is re-referenced before a real promotion or demotion takes the slot, the miss and the re-put cycle are both avoided.
- hit-rate (128K, `web_0`): This cell moved up only marginally, within noise. It recovers none of the giveback the parent took on from the probing-placement and ghost interaction.

## Hurt

None of the cells regressed. The real "hurt" is that the targeted cell did not move:

- hit-rate (16K, `web_0`): This is the cell the hypothesis aimed at, and it is effectively flat. When a stash bucket overflows at 16K, the victim's target and probing buckets are almost always already full. The idle normal-bucket capacity the hypothesis expected to find there is not there, so the rescue branch rarely fires and the victim still goes to the ghost.
- hit-rate (16K, `hm_0`): Unchanged, for the same reason. At the small size, normal buckets are saturated whenever the stash overflows.

## Takeaway

Rescuing stash-overflow victims only into free normal slots is safe: it never displaces a resident and adds no cascade. It only pays off where free slots actually coexist with stash churn, which means the larger cache, not the small one. That rules out the theory that `web_0` 16K is stuck because of idle normal-bucket capacity. That cell is capacity-saturated, so the next attempt should change which item gets evicted from a full bucket, not look for empty slots. One side effect for later changes to keep in mind: a rescued item takes up a target slot. That can push a target below `STASH_GATE_BYPASS_FREE_SLOTS` and turn later first-hit bypass promotions into gated ones.
