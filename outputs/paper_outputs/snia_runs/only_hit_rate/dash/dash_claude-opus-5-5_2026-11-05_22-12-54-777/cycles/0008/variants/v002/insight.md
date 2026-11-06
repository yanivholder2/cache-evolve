# Variant v002 Insight

## Helped

- hit-rate (hm_0, size 16384): This was the largest move in the variant, and the hypothesis did not predict it (it expected hm_0 to stay flat). At the small size the target tier is often full of warm entries. The gate now refuses the stash bypass when the 8-slot tail window has no `lfu_counter == 0` slot, so the `demote_from_normal_bucket` cascade stops pushing reused residents into the stash. On hm_0, which reuses more than web_0, protecting those residents is worth more than fast-tracking a returner.
- hit-rate (web_0, size 16384): A small gain in the predicted direction, near noise level. Gating on contention removes some of the swaps v002 made here, where a useful resident was traded for a one-time returner. The parent named this cell as its only flat spot.

## Hurt

- hit-rate (web_0, size 131072): This cell gave back part of v002's large-size gain. The premise that large targets usually have a cold tail slot does not fully hold on web_0. When the window is all warm, ghost-hit returners now take the cycle 0006 stash route again. Some of them get dropped from the stash before their next reuse, which is the failure v002 had fixed.

## Takeaway

Gating the ghost bypass on a cold slot in the tail window moves the bypass's benefit from large caches to small ones. It protects warm residents where the target is contended (the hm_0 and web_0 cells at 16384). But it also blocks some useful bypasses on web_0 at 131072, where tail windows can be all warm even in a big target. So "no cold slot in the window" is an imperfect sign that a returner is not worth admitting. For example, comparing the returner's credit with the coldest counter in the window might keep the protection and still let valuable returners through. hm_0 at 131072 stayed flat, as predicted.
