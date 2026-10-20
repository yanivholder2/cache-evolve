## Recurring wins

- Behavior-preserving rewrites that remove a mechanism-identified cost keep paying off, with hit rate unchanged. Examples: scan consolidation, hash index, O(1) ring ghost, tagged index, split ghost map, cheap hasher.
- The lever has moved step by step. Linear scans came first, then ghost maintenance, then hash-map traffic (probe count per `put`, hashing cost per `get`). Each fix exposed the next bottleneck.
- Cutting the number of map operations per `put` (the tagged index) was the largest single `put` gain. A cheaper hasher mainly helps `get`.
- A smaller small queue (10%, then 5%) gave small, directionally consistent hit-rate gains. They are the only hit-rate gains seen, and they flatten out.

## Recurring failures

- Anything that loosens main admission or eviction has lost hit rate or gained nothing. This includes a lower promotion threshold, a larger small queue, ghost recycling, a freq head start, a larger ghost bound, a rotation cap, and the recent-main second chance. The small queue's one-hit-wonder filtering is what matters.
- Micro-optimizations (frequency store, deque pre-sizing, CLOCK-style main eviction rewrite) land within noise. Main-queue eviction is not the cost.
- Latency shifts without a mechanism (`get` moving in variants that never touch it, big drops from constant-only edits) recur every cycle. They are noise, and they let neutral or regressing variants win selection.
- Speed wins carry hidden costs. Whole-cache footprint grows with each structure, and the per-entry metadata metric ignores heap. Bundled changes (pre-sizing with hasher) make regressions hard to attribute.
- Infrastructure failures (a Maven 429) can hide an idea without testing it.

## Open questions

- Do the gains hold across other traces, cache sizes, and repeated runs? All evidence is workload-a at one size.
- What is the real heap footprint of the tagged index plus the separate ghost map?
- Do the split ghost map and the cheap hasher combine cleanly? Was the `put` tail regression caused by pre-sizing or by weak hash mixing?
- What remains in the `put` tail after the map work? Candidates are eviction and the small-to-main move.
- Is there any hit-rate idea that does not loosen main admission? Is a main-evicted-key special case worth it as an O(1) tagged ghost?
- Were the tests deleted by the scan refactor restored?

## Per-policy notes

- S3-FIFO (the only policy so far):
  - The current best structure is a 5% small queue with baseline promotion (freq threshold and ghost sizing at baseline), a ring or split ghost, and a tagged hash index.
  - The baseline ghost size and promotion threshold look near a local optimum on workload-a.
  - Hit rate has been flat across cycles. Progress has been in `put` and `get` cost.
  - Judge on hit rate and on changes with a clear mechanism, not on raw latency.
