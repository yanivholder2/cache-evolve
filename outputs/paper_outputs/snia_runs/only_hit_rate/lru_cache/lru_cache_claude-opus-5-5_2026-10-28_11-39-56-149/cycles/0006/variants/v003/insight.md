# Variant v003 Insight

## Helped

None. Both cells at size 16384 came out identical to the parent. At that size `capacity.clamp(50_000, 400_000)` still gives 50,000, so the controlled no-op worked as designed. That also shows the change has no side effects outside the large size.

## Hurt

- hit-rate (web_0 @ 131072): A slight drop. The period went from 50,000 to 131,072 accesses, so the climber gets about 2.6x fewer rounds over the trace. `step_init` is still capped at 256 and decays 2% per round, so each round moves the window by at most a few hundred entries in a 131k cache. Fewer rounds therefore means the window ends up less far from its 1% starting point. web_0 is the trace that gained from window growth in v002.
- hit-rate (hm_0 @ 131072): A marginal drop from the same mechanism. Each direction decision now waits through a full cache turnover. As a result, the climber is slower to back out of a resize that hurt. The extra smoothing does not seem to have produced cleaner hit-rate comparisons that would make up for the slower response.

## Takeaway

How long a resize takes to settle depends on the step size, not the cache capacity. A move of at most 256 entries fills or drains in a few hundred misses, so even the old 50,000-access period was already long enough to judge it at 131072. Making the period longer while keeping the step capped only cuts the number of climbing rounds. At this step cap, the large size is limited by how many rounds it gets, not by measurement noise. Scaling the period only makes sense together with scaling the step (for example, a capacity-proportional step at the large size). On its own it does not help.
