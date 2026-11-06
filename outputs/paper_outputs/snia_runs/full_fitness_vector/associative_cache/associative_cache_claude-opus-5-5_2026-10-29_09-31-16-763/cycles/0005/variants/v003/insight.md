# Variant v003 Insight

## Helped

- hit-rate (hm_0 small cache, web_0 large cache): Small gains, close to noise. At the small hm_0 size, evictions are frequent enough that the 1-in-8 decay roughly balances the +1-per-hit growth, so depth moved a little above quarter depth and kept a few more recent admits alive. That is the only cell that behaved the way the hypothesis predicted.
- per-op-time: Get p50 eased slightly. The hit path only added a compare and a write to a field on the bucket header, which is already hot. The change is small enough to be noise and doesn't come from the policy change.

## Hurt

- hit-rate: The summary went down, and the cell the hypothesis was built for (hm_0 at the large cache size) fell the most. The hit signal `position < depth` can't tell a probationary item apart from an old MRU item that has aged down into the LRU zone. On a trace with many hits and few evictions, any near-eviction hit pushes depth up, while decay only runs once every 8 evictions. So depth likely drifts toward `DEPTH_MAX` (half the bucket). New one-touch keys then land in the middle of the stack and push out reused items that LIP's quarter depth used to protect.
- hit-rate (web_0 small cache): Scan resistance dropped a little, which fits the same feedback loop. Hits on aged items near the LRU end raise depth even during scan phases, so scan keys are admitted deeper than in the parent.
- per-op-time: Get p99 rose noticeably and put p50/p99 crept up. Possible causes: a branch the CPU predicts badly, plus a store to `depth` on hits at low positions, and a counter with a modulo on every LIP eviction. The tail move is larger than the parent's run-to-run jitter, so some of it is probably real. Separately, the bucket struct grew by four `u32` fields. `depth_min` and `depth_max` are stored per bucket instead of as constants as the hypothesis specified. This growth is not visible in the metadata-size dimension, which measures only `Item` and `AssociativeCache`.

## Takeaway

Where a hit lands in a single LRU-ordered bucket is not a usable sign of under-admission. Positions below the insertion point mix new admits with old items about to be evicted, so the controller mostly reads "the cache is under pressure" and responds by admitting deeper. That is the wrong response for both traces. An adaptive depth needs a signal that actually marks an item as probationary (for example a per-item "admitted, not yet promoted" bit) and symmetric grow/decay rates. Otherwise the fixed quarter depth remains the better choice.
