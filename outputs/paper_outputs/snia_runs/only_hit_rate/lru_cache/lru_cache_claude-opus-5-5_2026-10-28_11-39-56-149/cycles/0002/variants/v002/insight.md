# Variant v002 Insight

## Helped

- hit-rate (hm_0, both sizes): The count-min admission gate, comparing the window-LRU candidate with the probation-LRU victim, kept one-shot blocks from replacing probation entries that had been seen more often. hm_0's frequently reused blocks therefore lasted longer in the main area than under v001, where every new key entered probation unconditionally.
- hit-rate (web_0 @ 131072): This is the biggest gain in the variant. The 1% window holds roughly 1.3k entries at this size. That is enough to catch short-range re-references before admission runs. After that, the tie-admits rule (`cand_est >= victim_est`) lets blocks with equal frequency flow through as recency would. So web_0 kept its recency hits and also gained frequency filtering. This is the opposite of what the rationale says happened to v003's strict-greater gate.

## Hurt

- hit-rate (web_0 @ 16384): This is the only cell that slipped, and only slightly. At this size the window is about 160 entries, too small to absorb web_0's short-range reuse. The sketch is also sized near `capacity` and halves every `10 * capacity` increments, so it is narrow and resets often. As a result, more second-access candidates lose to a victim whose count is inflated by hash collisions or slightly higher, and they are evicted from the window. v001 would have kept them in probation. Also, `get` increments the sketch on a miss and `put` increments it again for the same new key. A missed-then-inserted key is therefore counted twice, which blurs the difference between keys that were never reused and keys that were reused once.

## Takeaway

On these MSR traces, the combination of a tiny admission-free window and tie-admitting TinyLFU in front of v001's SLRU is a net win. The window plus lenient ties removes the web_0 penalty that strict frequency gating causes, at least at the large size. The remaining weak spot is web_0 at small capacity. There the fixed 1% window and the capacity-sized, frequently halved sketch are probably too small. The next things to try are tuning window size (e.g. adaptive or larger than 1% at small sizes), sketch width, and reset period, and keeping the admission rule as it is. Fixing the double increment on miss-then-put is worth testing alongside those.
