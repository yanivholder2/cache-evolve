# Variant v002 Insight

## Helped

- metadata-size: Halving sketch depth from 4 to 2 reduces the CountMinSketch heap footprint from 65536 to 32768 bytes (exact halving).
- per-op-time (get, p50): Fewer hash rows (2 instead of 4) cut per-operation CPU work, improving get median latency from 132ns to 117ns.

## Hurt

- hit-rate: Reducing independent hash functions from 4 to 2 increases collision probability in the count-min sketch, degrading frequency estimation accuracy and harming admission decisions; hit-rate dropped from 73.6% to 72.4%.
- per-op-time (get, p99): Median improved but tail latency exploded (171ns → 376ns), suggesting the deeper problem is worse admission decisions driving adverse cache states rather than per-operation cost.
- per-op-time (put): Put latency regressed at both p50 (166ns → 185ns) and p99 (400ns → 435ns), indicating that increased cache evictions from poor frequency estimates overwhelm any per-operation efficiency gains.

## Takeaway

Reducing sketch depth trades per-operation work for frequency estimation accuracy, but the tradeoff is severely negative. The lower hit-rate and worse admission decisions cause more evictions and puts overall, eroding the latency benefit on the median path and harming tail latency. The 50% memory savings do not justify the degraded cache behavior.
