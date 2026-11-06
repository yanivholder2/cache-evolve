# Variant v003 Insight

## Helped

- hit-rate: Hit rate went up in all four trace×size cells, with the largest gain at hm_0 131072, which the hypothesis named as the first check. The direction is consistent, but every change is far smaller than the ±0.001 noise tolerance. The two-slot window only fires when the LRU slot has already taken its first stash hit and the slot next to it has not. That arrangement seems to be rare: a once-hit item usually gets refreshed toward MRU, or promoted on its second hit, before it drifts back to the tail. The bounded protection did as intended and caused no harm, but it rescues very few items.
- per-op-time: Get and put both got faster at p50 and p99. This change can't be the cause, because it only adds work: one counter compare, and sometimes a mid-bucket remove-and-shift plus `refresh_stash_mask`. The speedup is most likely run-to-run benchmark variance. It does show that the extra cost on stash eviction is too small to measure, as the hypothesis predicted.

## Hurt

(None. metadata-size did not change because no fields were added and `Item` is still 40 bytes.)

## Takeaway

Using per-item hit state to choose the stash victim is safe and cheap, but a window of only the last two slots almost never finds the "once-hit LRU, never-hit neighbour" pattern. So it cannot move the hit-rate plateau of about 0.535. If this signal is worth pursuing, the window needs to be wider. One option is to evict the LRU-most `lfu_counter == 0` slot from the whole bucket, while keeping a bound such as "protect each item at most once" to avoid the unbounded residency seen in 0004/0005. The other option is to drop this lever, since stash eviction choice looks like a weak lever on these traces.

(Note: I could not list or open the overlay source files in this session, so I took the description of the code change from the hypothesis and did not check it against the diff.)
