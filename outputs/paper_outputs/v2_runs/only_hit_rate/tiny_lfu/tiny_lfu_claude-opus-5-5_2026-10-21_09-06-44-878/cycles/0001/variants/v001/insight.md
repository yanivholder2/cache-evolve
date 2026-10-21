# Variant v001 Insight

## Helped

- hit-rate: Only marginally. The 1% window now lets a newcomer stay resident long enough to be re-referenced, and probation→protected promotion keeps keys that were hit twice off the victim position. But at size 2048 the window is only ~20 slots, so the change rescues just the few keys that come back within ~20 inserts. Apart from that, the main region still gets the same frequency-gated admission as the baseline.

## Hurt

(No measured dimension regressed. v001's fitness vector reports only hit-rate, so metadata-size and per-op-time could not be compared with the parent. A regression in either is plausible, but it was not measured: the new `seg: u8` tag adds per-node bytes, and the window-overflow path now does three-list bookkeeping on every miss.)

## Takeaway

On workload-a, switching from TinyLFU+LRU to the canonical W-TinyLFU layout gave essentially no hit-rate gain. What still limits it is the admission contest. That contest compares raw `u32` count-min estimates that are never halved or aged, so the candidate is judged against stale accumulated history no matter how it reached the contest. In this codebase, segmentation by itself doesn't pay off. The next lever is probably sketch aging/reset (with a larger or adaptive window), not further restructuring of the resident lists.
