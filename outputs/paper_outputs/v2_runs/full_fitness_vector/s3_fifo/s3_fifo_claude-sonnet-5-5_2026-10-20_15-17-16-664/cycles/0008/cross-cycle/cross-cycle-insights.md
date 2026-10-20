## Recurring wins

- Behavior-preserving rewrites with a clear mechanism are the reliable gains: removing linear scans, hash-index membership, ring-buffer ghost, and the single location-tagged index with a split ghost map. Each cut a specific cost and left hit rate identical.
- The `put` tail was traced step by step. Linear scans came first, then ghost maintenance, then hash-map probe count. Fixing the real bottleneck moved it. Fixing something adjacent did not.
- Shrinking the small queue helped hit rate with diminishing returns (10% → 5% → 3%). It is the only hit-rate lever that has paid off.
- A cheap hasher helps `get` on the tagged index.

## Recurring failures

- Anything that loosens main admission lowered hit rate or gave nothing: freq>=1 promotion, a larger small queue, ghost recycling, a freq head start, a larger ghost bound, and the recent-main second chance. The small queue's filtering role matters more than the loosened variants assumed.
- Main-queue eviction rewrites and rotation caps gave nothing. Eviction-policy tweaks have little room on this trace.
- Micro-optimizations (frequency store, deque sizing) and hasher/finalizer/pre-size tuning ended in tradeoffs with no free win. That direction is exhausted.
- Latency shifts with no mechanism recur, including `get` moves in variants that never touched `get`. Winners picked on such shifts are probably noise. Latency alone is an unreliable selection signal.
- Bundled changes (pre-size plus finalizer) made it impossible to attribute effects.

## Open questions

- Do the gains hold across other traces, cache sizes, and repeated runs? Every result so far is one trace at one size.
- Is the 3% small ratio safe on workloads with more mid-reuse objects?
- What is the real heap footprint of the tagged index plus split ghost map? The metadata metric counts only inline struct bytes.
- Can hit rate improve through promotion or ghost policy changes that do not loosen main admission? One option is an O(1) tagged variant of the ghost.
- What remains in the `put` tail, and can the `get` p99 cost of the tagged index be reduced?
- Were the tests removed in the scan refactor ever restored?

## Per-policy notes

- S3-FIFO (the only policy tried): the small queue is a filter for one-hit-wonders. Keep the baseline promotion threshold, keep main admission strict, and keep the ghost near its baseline size.
- The current base is the tagged index, split ghost map, ring ghost, and one Fx-style hasher with moderate slack. Stop work on hashers and ratios.
- Next cycles should seek hit-rate ideas, validate on multiple traces with repeated runs, and credit only mechanism-backed latency changes.
