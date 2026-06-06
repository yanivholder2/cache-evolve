# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The amortized right-shift halving in `CountMinSketch::reset` decays the frequency signal on this trace faster than it sharpens it — on workload-a the hot set is stable enough that aging mostly discards earned counts for persistently-hot keys, weakening admission/eviction contrast rather than refreshing it, so hit-rate fell below the parent's un-aged sketch.
- per-op-time: Adding `additions`/`reset_at` bookkeeping to every `increment` plus the periodic O(width) sweep raised both `get` and `put` latency (p50 and p99 up across the board); the sweep cost amortizes but is not free, and the per-op counter bump now runs on every access.

## Takeaway

On a workload with a stable hot set, classic TinyLFU counter aging is a net loss even when tuned wide (interval = sketch width, single amortized sweep): the parent's monotonically-inflating counters were not the bottleneck here, so halving them throws away discriminating signal and adds per-op cost for no hit-rate return. Aging only pays off against genuinely shifting hot sets, which workload-a does not exhibit.
