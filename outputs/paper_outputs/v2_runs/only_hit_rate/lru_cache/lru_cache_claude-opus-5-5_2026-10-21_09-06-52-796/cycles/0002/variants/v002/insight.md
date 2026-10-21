# Variant v002 Insight

## Helped

(No measured dimension improved. As with v001, `hit-rate` on LIRS workload-a at size 2048 is the only dimension in the fitness vector.)

## Hurt

- hit-rate: The ARC-style feedback is biased toward growing `small`. The small ghost keeps v001's cap of `capacity - initial_small`, which is close to the full capacity, while the new main ghost is capped at only `capacity / 2`. In workload-a's loop phases, keys cycle out of probation and come back while they are still in the large small ghost, so "grow" signals far outnumber "shrink" signals. Meanwhile the `len/len` step ratio rounds to 1 in both directions and cannot correct the imbalance. `small_target` most likely drifts toward `max_small = capacity/2`. That halves the frequency-protected `main` FIFO and lets loop and scan blocks take up to half the cache. v001's fixed 10% probation was designed to prevent exactly that.
- hit-rate: In S3-FIFO, a small-ghost hit already has a remedy: the key is admitted straight into `main`. v002 also reads that hit as "probation too short" and enlarges `small`, so the same event is corrected twice. The admission shortcut and the queue-resize signal pull in the same direction instead of balancing each other.
- hit-rate: Main-ghost hits now skip probation and go directly into `main` with `freq = 0`. This adds a second fast path into `main` that doesn't require proving reuse while resident. That weakens the admission filter that v001's insight credited for its gain.

## Takeaway

A direct ARC transplant onto S3-FIFO hurt hit rate on workload-a. In S3-FIFO, a small-ghost hit means "this key deserved `main`", and ghost-hit admission already handles that. It does not mean "probation should be larger". Combined with asymmetric ghost capacities, the feedback loop ends up growing the scan-exposed queue. This answers cycle 1's open question about sensitivity to the small-queue size: hit rate is sensitive to it, and moving away from the small ~10% split hurts. A future adaptive variant would need equal ghost capacities and the opposite signal mapping, or it should keep `small_target` fixed and tune it offline.
