# Variant v002 Insight

## Helped

- hit-rate: No loss, which supports the hypothesis's "no measurable hit-rate loss" claim. Every trace and size matches the parent exactly, including hm_0 at 131072. The skip only leaves an item one slot short of MRU, and eviction takes from the LRU end, so the victim choice almost never changes.

## Hurt

- per-op-time: `get` p50 is flat, so the main claim fails. Skipping the shift on near-MRU hits did not speed up the median hit. `get` p99 got noticeably worse, and the only new code on that path is the `i + 2 >= len` compare-and-branch. Two causes are plausible: the branch mispredicts on mixed near-MRU and deep hits, or the run was noisy. This needs a rerun before blaming the change. `put` did not move, as expected, because `put` and eviction were left alone.

## Takeaway

The per-hit MRU shift does not cause the `get` p50 gap. Removing it on the hottest hits left the median flat. Cycles 3, 4 and 5 already ruled out scan order, scan direction and the target→probing cascade, so the extra stash probe on the normal hit path is the remaining suspect. The skip-reorder guard brings no p50 gain and may cost p99, so it should be dropped instead of carried forward. The next hypothesis should target the stash probe directly, for example by checking the stash only after both normal-tier buckets miss.
