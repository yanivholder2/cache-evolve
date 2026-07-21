# Variant v002 Insight

## Helped

- **hit-rate**: Halving `prob_cap` from `capacity/8` to `capacity/16` expands the protected quota from 87.5% to 93.75%, allowing more moderately-frequent Zipfian items to remain shielded from eviction and producing a small but real incremental gain over the parent.

## Hurt

- **per-op-time**: The batch-2 demotion mechanism failed to hold the get p50 at the parent's (anomalously low) 0 ns floor; with a narrower probationary window, more get operations trigger a probationary→protected graduation, and when those graduations land on a deferred batch event the combined unlink+push_back×2 cost pushes the median get above 100 ns.

## Takeaway

Tighter probationary quotas continue to improve hit rate in a monotone fashion, but the batch-2 demotion scheme did not reduce per-op latency — it doubled the per-event cost while the smaller probationary buffer increased graduation frequency, leaving the net demotion burden on the hot get path unchanged or worse. The hypothesis that halving event frequency cancels doubling event magnitude does not hold when the quota change itself raises baseline graduation rates.
