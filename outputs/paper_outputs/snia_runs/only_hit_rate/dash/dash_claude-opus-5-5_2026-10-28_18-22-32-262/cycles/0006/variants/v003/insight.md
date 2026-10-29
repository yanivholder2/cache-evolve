# Variant v003 Insight

## Helped

- (none) No cell went up. The ghost-backed demotion credit did not turn into extra hits on any trace or size.

## Hurt

- hit-rate (128K, `web_0`): This cell lost the most, and it is also where the parent's put-time ghost credit gained the most. Under the parent, a key that came back used up its fingerprint on `put`. Now the fingerprint stays, so the same returning key also gets a one-hit pass back into its target when it is later demoted. In a full 128K target, that fast return pushes out another resident, which then goes down through the stash. That resident is usually just as hot, so the swap costs about as many hits as it saves.
- hit-rate (16K, `web_0`): Small drop. The parent's insight already found that the ring reaches only a short span of evictions here, so few demotions find their fingerprint. When one does, the fast return goes into a small, crowded target. It is a weaker version of the cycle 0003 v001 failure, where demoted items got a one-hit return.
- hit-rate (16K, `hm_0`): Small drop, same mechanism. The keys whose re-put helped under the parent are the same keys that now get the demotion shortcut. Their second trip back into the target displaces residents that had earned their slots under the two-hit gate.
- hit-rate (128K, `hm_0`): Flat. Demotions of keys with ghost history are rare here, or they are neutral. That fits `hm_0`'s gain under the parent coming from the re-put itself, not from what happens after the key is demoted.

## Takeaway

A ghost fingerprint is good evidence that a key will be reused when the key comes back, but it is not evidence that the key should beat its target's current residents after it has already lost that slot. Using the same evidence twice gave no gain anywhere and cost a little on `web_0`. So consuming the fingerprint on `put`, as the parent did, was the right lifecycle, and the "should demotions also check the ghost?" question can be closed as no. Three side effects of the change were not separated from the main one, and may account for part of the small loss: skipping duplicates in `put_stash_recording`, fingerprints that now survive a re-put, and the extra ghost scan on each demotion.
