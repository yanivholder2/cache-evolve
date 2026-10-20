## Recurring wins

- Mechanics changes on the plain LRU (u32 links, lighter/folded Fx hasher, head-touch early return, move-based eviction instead of key clone) gave speed and/or size without touching hit-rate. Hit-rate came only from policy; speed and size came only from mechanics. The axes are independent.
- Segmented LRU is the one proven hit-rate lever (scan resistance). It is established; no need to re-prove it.
- Combining independent winners (segmentation + fast hasher + cheap touch + clone removal) worked in cycle 0002, with put gaining most.
- Packing the segment bit into an existing link field keeps per-node size flat.

## Recurring failures

- Once the policy is segmented, link-width and bit-packing changes stop bringing speed. They shrink size at best, and add widening/NIL/mask work to every link access.
- Segmentation's real cost is promotion/demotion relinking on the hot path. Moving it between get and put (lazy demotion) or packing flags only relocates the cost. Median gains have often come with worse tails (put p99 from relinking, get p99 from weak hashing).
- Bundling several changes (fold + layout, hasher + touch + clone) makes attribution guesswork, since authors cannot read diffs. This blocked conclusions in cycles 0002 and 0003.
- Per-node size metrics hide container growth (extra head/tail pair).
- Infrastructure failures (build rate limit) wasted a slot in cycle 0001. They say nothing about the idea's merit.

## Open questions

- Is the hash fold or the u32 conversion responsible for the slowdown seen on segmented u32 variants? Test them in isolation.
- Can promotion be made cheaper by skipping the relink for nodes near the head, or by sampling or batching it?
- Can the segment be stored implicitly (list membership) so no masking is needed?
- Does the hit-rate gain justify the latency cost on traces other than the scan-heavy one?
- Does the fold fix get/put tails on a segmented policy? This is unconfirmed.

## Per-policy notes

- Plain LRU: the mechanics wins are mostly exhausted. u32 links, the folded Fx hasher and move-based eviction were speed and size wins.
- Segmented LRU: good hit-rate on scans, but it costs per-op time through promotion/demotion. The cycle-0002 merged candidate (segmented, u32 links, folded Fx, move-based eviction) is the baseline. New variants should recover speed without losing hit-rate.
- Segmented plus u32 links: size gain only, with no speed gain observed so far.
