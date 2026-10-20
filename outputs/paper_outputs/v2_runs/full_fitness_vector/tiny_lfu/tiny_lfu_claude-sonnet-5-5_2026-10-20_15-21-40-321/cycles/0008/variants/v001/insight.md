# Variant v001 Insight

## Helped

- metadata-size: Halving the sketch counter table cut the heap footprint of the sketch by half, as intended. The Node and cache struct sizes are unchanged, so the summary metric did not move.

## Hurt

- per-op-time: Both get and put latency, p50 and p99, got worse, which is the opposite of the hypothesis. The get p99 tail was hit hardest.
- hit-rate: Hit-rate fell slightly on workload-a. The drop is within the hypothesis's 0.002 refutation bound, but it is a real cost of the narrower table. More counters collide, so frequency estimates are noisier.

## Takeaway

The parent's 64 KiB sketch was not the cache-footprint bottleneck for put/get latency. Shrinking it to 32 KiB bought no locality benefit and made latency worse. The likely cause is that the narrower table (halved width, with the aging threshold rescaled) changes collision and reset behavior and adds work on the hot path. The earlier "width increases don't help hit-rate" finding does not imply that smaller is faster. The next latency attempt should target something other than sketch size.
