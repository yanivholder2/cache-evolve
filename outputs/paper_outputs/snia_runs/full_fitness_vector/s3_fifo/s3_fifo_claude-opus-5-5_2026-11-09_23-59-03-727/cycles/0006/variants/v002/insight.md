# Variant v002 Insight

## Helped

- hit-rate: At the small size (16384), web_0 and hm_0 both ticked up by a tiny amount. This fits the hypothesis: a key with one hit that reaches the small tail gets one more pass instead of going to ghost. When small is short, that extra pass is sometimes long enough to catch the second hit that graduates the key. But the gain is too small to separate from noise, and it moved the summary only marginally.

## Hurt

- hit-rate: At the large size (131072), web_0 and hm_0 both slipped slightly. web_0 at 131072 is the exact gap the hypothesis meant to close. A likely reason is that an extended key holds a small slot for a second full pass, so new freq-0 entries get pushed out sooner. Probation gets shorter for newcomers in exchange for longer probation for keys with one hit. Also, the old demotion path was already a second chance: a demoted key goes to ghost, and a ghost hit sends it straight to main. So the extension mostly duplicates what the ghost queue was already doing.
- per-op-time: `put` p99 roughly doubled, and the other latencies rose slightly. The new `evict_small()` loop can pop and re-push many entries in one miss: every unextended freq-1 entry in a run at the tail gets recycled before one slot is freed, with a worst case near `small.len()` iterations. The old code always did one pop per eviction.

(metadata-size did not move: the new `extended: bool` fits in the existing padding, and `Entry<u64,u64>` stays at 24 bytes, as `entry_fits_in_padding` asserts.)

## Takeaway

A reinsertion into small, limited to keys with one hit, does not close web_0's gap at the large size. It gives back a little large-size hit-rate and adds a clear `put` tail-latency cost from the multi-pop eviction loop. The ghost queue already gives demoted freq-1 keys a path back into main, so web_0's large-size gap is probably not about keys with one hit leaving small too early. Further probation changes should look at how the ghost hands keys to main (its size or admission rule) instead of adding more rounds inside small.
