## Recurring wins

- Mechanics changes (lighter/folded Fx hasher, head-touch early return, move instead of key clone on eviction, u32 links) improve speed and/or size and leave eviction order, and so hit-rate, unchanged. Speed/size and hit-rate are independent axes.
- Segmented LRU is the proven hit-rate lever on scan-heavy traces. It is established and needs no more proof.
- Bundling segmentation with the mechanics wins worked (cycle 2 combined variant won, no hit-rate regression), and put benefited most.
- An avalanche fold on the hash seems to avoid the get p99 regression of the weak hash, but this is unconfirmed on a segmented variant.

## Recurring failures

- Median-latency gains that cost tail latency: weak hashing hurt get p99, and promotion/demotion list surgery hurt put p99. Cycle 2 reduced the hash part but the promotion/demotion tail is still open.
- Segmentation adds metadata (bigger node, extra list bookkeeping). Packing the segment bit recovers the size but adds mask/accessor cost on every link access.
- Infrastructure failures (a build rate limit) can reject a variant without testing it. They say nothing about the idea's merit.
- Bundled changes make attribution inferred rather than measured, because authors can't see diffs.

## Open questions

- Do u32 links and segment-bit packing compose, given the spare-bit budget? Can the segment be stored implicitly, for example by list membership?
- Does the hash fold fix both get and put tails on the segmented policy?
- Which bundled piece (hasher, touch fast path, clone removal) carries the speed gain?
- Does the segmentation hit-rate gain hold on non-scan traces?
- Can the promotion/demotion put-p99 tail be reduced?

## Per-policy notes

- Plain LRU (baseline): u32 sentinel links with a hash fold won on speed and size, with hit-rate unchanged. Fast hasher plus touch fast path also won.
- Segmented LRU: hit-rate gain from scan resistance. The costs are metadata size and put p99 from promotion/demotion. The next step is a merged candidate: segmented + u32 links + folded Fx + head-touch + move eviction, with the segment bit packed only if the masking stays off the put path.
