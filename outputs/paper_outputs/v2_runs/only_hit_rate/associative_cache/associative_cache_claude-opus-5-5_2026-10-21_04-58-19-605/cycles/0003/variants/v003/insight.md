# Variant v003 Insight

## Helped

- hit-rate: In the full-bucket path of `put`, one line changed: `self.hand = idx` replaces `self.hand = idx + 1 (wrapping)`. The GCLOCK hand now stays on the newcomer it just inserted. On the bucket's next miss, the sweep looks at that newcomer first. If the newcomer was not hit in the meantime, its counter is 0 and it is evicted at once. Before this change, it got a full revolution of grace. The result is a LIP-style filter: a key used once is pushed out after a single miss, and the established keys with non-zero counters are no longer worn down by sweeps whose only job was to make room for it. Hit rate rose over the parent and moved just past the v001 figure quoted in H03, so the hypothesis held. Its stated refutation, a hit rate below the parent, did not happen.

## Hurt

(No measured dimension regressed. Only `hit-rate` is in the fitness vector. The change should also make the common miss path cheaper, since an unhit newcomer is evicted in O(1) without a sweep, but `per-op-time` was not measured, so this is untested. The lead over v001 is also thin, and it comes from a single trace at a single size, workload-a at 2048. The claim that "the one-miss grace window is not too short" is so far supported only on that one trace.)

## Takeaway

The parent's GCLOCK counter had closed only part of the gap to v001. The rest of the gap came from where the hand rests after an insertion, not from frequency. Leaving the hand on the newcomer turns CLOCK's implicit full-revolution grace into a filter for keys used once, and that filter is aggressive. It is the same lever that made deep insertion win. With it, the in-place, no-shift CLOCK design matches or slightly beats v001 on workload-a. The next things to test are traces with longer reuse distances, where a grace window of one miss may evict keys that would later be reused, and per-op latency, to confirm the cheaper miss path.
