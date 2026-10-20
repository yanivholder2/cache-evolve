# Variant v001 Insight

## Helped

- metadata-size: No change. The sampled counters fit in existing padding, as the hypothesis intended, so the footprint stayed flat. This is neutral rather than a gain.

## Hurt

- hit-rate: Switching to a deeper len/2 insertion point when the sampled hit ratio is high did not beat the fixed len/4 insertion of the parent. The 1-in-16 sampled counters, evaluated every 1024 puts against a fixed 0.5 threshold, probably flipped the mode on workload-a in a way that weakened scan resistance. I could not read the source, so this is inferred from the hypothesis and the vectors.
- per-op-time: Both get and put got slower than the parent. The sampled-bucket branch in the get path and the periodic check and decay in the put path were meant to be near-free, but they added measurable cost. The extra branch and counter updates likely also hurt the hot path's code layout and branch prediction.

## Takeaway

Sampling the hit/miss counter did not make adaptive insertion depth free: it cost latency on both operations and lost hit rate against the fixed len/4 insertion. On this workload, the parent's static position-only policy was already near a local optimum, and a coarse global hit-ratio signal was not a good enough switch to improve on it.
