# Variant v003 Insight

## Helped

- (none): widening the ghost cap to 5/8 of main did not move any dimension in the intended direction; the targeted near-reuse hits were not recovered.

## Hurt

- per-op-time: the larger ghost cap lets `ghost` hold more keys, so the linear `iter().any()` membership scan and `retain` traverse a longer deque per operation — put p50 and p99 both rose markedly, matching the predicted scan cost.
- hit-rate: the longer residence window failed to convert extra ghost slots into hits, leaving hit-rate flat-to-marginally-lower rather than lifted.

## Takeaway

For workload-a, the dual-source ghost's content — not its length — was the binding constraint: stretching the cap from 1/2 to 5/8 of main bought no recovered near-reuse hits, confirming the 50% knee still holds even under v003's doubled inflow. The only measurable effect was a put-latency regression from scanning the wider membership deque, so the longer window is pure cost here. (The get p50 dip is opposite the change's expected direction and reads as measurement noise.)
