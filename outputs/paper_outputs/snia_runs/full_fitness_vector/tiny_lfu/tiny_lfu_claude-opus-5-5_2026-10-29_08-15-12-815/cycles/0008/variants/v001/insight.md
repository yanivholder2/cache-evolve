# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 131072): Most of the large-size web_0 loss that the hypothesis attributes to cycle 0007 v003 does not show up here, though this cell still sits a little below the parent. Here the dedup is kept, and `tick()` still moves `additions` forward on a matched missed-`get`→`put`, so the aging clock runs at the parent's rate. That points to the slower clock as the main cause of that loss. The small remaining gap below the parent is what the dedup itself costs.
- hit-rate (hm_0 @ 131072): This cell rose slightly. A newcomer's first access now counts once instead of twice, so window candidates arrive with one less unit of frequency. Under the strict `cand_freq > victim_freq` test they displace established probation entries less often, and at this size, where hm_0 is near its ceiling, that slightly better protects the resident working set.
- per-op-time (get p99): The get tail tightened while p50 held. The hypothesis kept the parent's `?` early-return shape and added a single unconditional `self.last_get_h = h;` store with no new branch. That adds nothing to the hit path, which fits the hypothesis's claim that v003's latency cost came from its two-armed `match` writing `self`, not from tracking the last get.

## Hurt

- hit-rate (hm_0 @ 16384): This is the cell the hypothesis expected to gain, and it moved the wrong way, below the parent. The parent's double count on miss→put worked as a bonus for keys missed again and again. A key that keeps getting evicted and coming back picks up an extra count each time, which helps it win the admission contest in a small cache. The dedup removes that bonus, so recurring keys at the small size lose ties they used to win.
- hit-rate (summary): The aggregate came in slightly below the parent. The small-size hm_0 loss and the leftover web_0 large-size gap outweigh the small hm_0 large-size gain. web_0 @ 16384 did not really move.
- metadata-size (TinyLfuCache struct): The struct grew by one `u64` for `last_get_h`. The node stays at 24 bytes and the sketch geometry is unchanged, so the summary does not move.
- per-op-time (put p50): The extra compare and branch on `key_h == self.last_get_h` in `put` adds a marginal cost to the put median. Put p99 did not get worse.

## Takeaway

Splitting v003's dedup from its aging-clock side effect shows the dedup by itself is not a hit-rate win on this line. With the clock back at the parent's rate, the large-size web_0 loss mostly goes away, but the hoped-for small-size hm_0 gain also turns into a loss. So the dedup's real effect is to remove a useful "miss-again" frequency bonus that the parent's double count gave to recurring, evicted keys. Whatever helped v003 at the small size was not the dedup. It was the slower aging, or the tie-break that H02 tests. Either way, the miss→put double count is better kept than removed.
