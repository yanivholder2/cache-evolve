# Variant v002 Insight

Note: I could not list or read the overlay source in this session (no file-listing or shell tool was available). This insight comes from the hypothesis and the two fitness vectors, not from a code diff.

## Helped

- (none) No dimension improved. `metadata-size` summary and `per-op-time` p50 for both ops stayed level with the parent.

## Hurt

- hit-rate: The doorkeeper makes the first sighting of a key invisible to the counters. That is meant to separate one-hit wonders from repeat keys. On workload-a the frequencies are already stable and well separated, so the filter gave no new signal. It cost a little accuracy, probably because every key's estimate is shifted by one and the counters lag by one access, which distorts the strict `incoming > victim` comparison.
- per-op-time: The extra bit test and set on every `increment` and `estimate` added work on the hot path. The `get` p99 and the `put` p99 both got noticeably worse, so the cost shows in the tail rather than the median. The likely cause is an extra random access into a second heap bitset, which adds cache misses.
- metadata-size: The `TinyLfuCache` struct grew in the per-type breakdown, which means the doorkeeper Vec header ended up inline in the struct. The hypothesis said to box it if that happened. The `Node` summary and the sketch heap figure are unchanged.

## Takeaway

A one-bit doorkeeper in front of the count-min sketch did not give a better frequency signal on workload-a. It slightly lowered hit-rate and widened the latency tail, and it grew the cache struct. The sketch was not the bottleneck on this workload, and the `+1` door correction doesn't help the strict-greater admission test when all keys are shifted the same way. Changes to sketch accuracy alone look exhausted here.
