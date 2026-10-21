# Variant v003 Insight

## Helped

- hit-rate: The 1-bit reference flag became a saturating 0..=3 counter in `Item::lfu_counter`. Each hit now adds to it, and each pass of the hand takes one away. A key hit many times can therefore survive up to three sweeps, while v002 gave it only one second chance. Newcomers still enter at 0 and are evicted first. Hit rate rose over v002, so per-key reuse frequency carries information on workload-a that a single recency bit throws away.

## Hurt

(No measured dimension regressed. However, the hypothesis's own success bar was clearing v001, and hit rate stayed below the v001 figure quoted in H03. So frequency recovered part of the gap between v002 and v001, but not all of it. `per-op-time` and `metadata-size` are still missing from the fitness vector. That leaves untested the hypothesis's claim that cost is unchanged: hits are still a single store, and `Item` does not grow. Misses can now take up to `CLOCK_MAX * len + 1` steps of the hand, compared with `len + 1` before, and that miss-path cost has not been measured.)

## Takeaway

Inside a short bucket, counting a few levels of frequency (GCLOCK, cap 3) beats a single CLOCK bit. Insertion stays the same as v002 (counter = 0), so the gain comes from protecting the hot set over several sweeps, not from filtering out keys used once. The gain is partial, though: v003 still trails v001 on the figure quoted in H03. A recency signal is still missing, or the cap of 3 is too low for workload-a's hot set. Note that this contradicts the parent insight's claim that v002 beat LRU: H03 puts v002 below v001. Changing `CLOCK_MAX`, or adding recency on top of the counter, are the obvious next levers. The miss-path latency cost of the longer sweep should also be measured.
