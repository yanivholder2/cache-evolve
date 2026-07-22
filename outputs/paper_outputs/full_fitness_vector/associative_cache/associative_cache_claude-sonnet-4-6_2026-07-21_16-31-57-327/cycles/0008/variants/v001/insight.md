# Variant v001 Insight

## Helped

- **hit-rate**: Cutting the scan window from 8 to 4 positions had virtually no effect on hit quality, confirming the hypothesis that never-accessed cold items are densely concentrated in the final 1–2 tail positions and are found just as reliably within a 4-item scan.

## Hurt

- **per-op-time**: Halving the scan depth increased p99 latency for both get and put operations, suggesting the narrower window causes more frequent fallback to the positional LRU tail (len−1), which in turn produces occasional slower eviction paths that inflate tail latency rather than eliminating the overhead the hypothesis targeted.

## Takeaway

Reducing the second-chance scan from 8 to 4 positions falsifies the hypothesis that scan depth was the source of the per-op-time regression: p99 latency worsened for both operations while p50 and hit-rate were essentially unchanged. The scan itself is cheap enough at 8 items that halving it produces no measurable median speedup; instead, the narrower window degrades eviction selection on the rare cases where no cold item sits in the bottom 4 positions, introducing tail-latency spikes. The regression source on the PUT path remains unaccounted for by scan depth.
