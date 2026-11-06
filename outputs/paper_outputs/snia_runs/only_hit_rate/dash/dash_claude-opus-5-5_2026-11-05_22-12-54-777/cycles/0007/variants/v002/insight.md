# Variant v002 Insight

## Helped

- hit-rate (hm_0, size 131072): In `DashSegment::put`, a ghost-hit key now goes through `promote_to_target(..., Front)` with `lfu_counter = 1`. It no longer goes back into the 8-bucket stash. Long-gap returners are no longer dropped from the stash again before their next reuse, so the ghost credit becomes target-tier residency. This is the effect the hypothesis predicted. It is the largest hm_0 gain the ghost line has produced: changing where the credited key lands did what changing ghost reach (c0005) and credit size (c0006) did not.
- hit-rate (web_0, size 131072): Gained, which the hypothesis did not predict (it expected web_0 to stay flat because its ghost rarely fires). At the large size the ghost table is big enough to catch some of web_0's returners. Sending them straight to target also makes up for v003's small regression here. In v003 these items had to re-earn residency through the stash. Now they skip that step.
- hit-rate (hm_0, size 16384): A small gain. The mechanism is the same, but the effect is weaker because a smaller target tier evicts a counter-1 Front arrival sooner. Fewer ghost-admitted keys survive in target long enough to be hit again.

## Hurt

- hit-rate (web_0, size 16384): Flat, at noise level. When the ghost does fire here, the key lands in a small, contended target and forces a `demote_from_normal_bucket` cascade. The resident it pushes out has its counter halved into the stash. On a scan-heavy trace this sometimes swaps a useful resident for a returner that does not come back again, so the change has no net effect.

## Takeaway

Where ghost-credited keys are placed mattered more than how much credit they get. Sending ghost hits straight to target Front with counter 1 (the ARC/2Q rule) gave consistent gains at the large size on both traces, while the earlier tuning of ghost reach and credit did not. Landing at Front with a low counter limits the risk: a ghost-admitted key that is not re-hit is evicted quickly, so at the small size the change is neutral rather than harmful. The remaining limit is target-tier capacity at 16384, not the stash bypass.
