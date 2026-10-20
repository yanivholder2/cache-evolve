## Recurring wins

- Behavior-preserving rewrites that remove a concrete cost with a clear mechanism are the only reliable wins. Examples: removing linear scans, a hash index for membership, an O(1) ring ghost, a tagged single index, a split ghost map, and a keyed single lookup. Hit rate stays identical.
- Hash-map traffic was the real `put` bottleneck. Cutting the number of map probes per op moved the tail. Ghost maintenance mattered before that. Queue manipulation and main-eviction rewrites did not.
- Lowering the small-queue ratio (10% → 5% → 3%) gave small hit-rate gains in the same direction each time. The gains shrink with each step, so the lever is saturating.
- Tail-latency gains appear when an O(n) step is removed. Medians barely move.

## Recurring failures

- Anything that loosens main admission lowers hit rate on this trace. This includes a lower promotion threshold, a larger small queue, a freq head start for ghost re-admits, a larger ghost, recycling main evictions into the shared ghost, and a lower frequency cap. The small queue's one-hit-wonder filter matters most.
- Eviction-policy tweaks give almost nothing: the rotation cap, the recent-main second chance, and adaptive small sizing. Their signals rarely fire here.
- Micro-optimizations (frequency store, deque pre-sizing, hasher swaps, finalizers) are exhausted. They trade median against tail with no free win.
- Selection through noisy latency keeps promoting Winners that have no mechanism. Latency shifts on paths a variant never touched are noise. Such Winners (several in cycles 1–9) should not be credited.
- Bundling changes (pre-size plus finalizer, for example) keeps hiding which part caused an effect.

## Open questions

- Do the gains hold on other traces and cache sizes? Every result comes from one trace (workload-a) at one size.
- Do latency gains survive repeated runs? The noise floor has never been measured.
- What is the real heap footprint of the index, split ghost map and ring? The `metadata-size` metric shows only inline struct bytes.
- Is the keyed single lookup (Cycle 9) compatible with the tagged index, split ghost map and Fx hasher, or does it duplicate them?
- Does 3% (or 5%) small ratio hurt workloads with more mid-reuse objects?
- Can a promotion or ghost-policy idea raise hit rate without loosening main admission? One option is an O(1) tagged variant of the existing ghost.
- What is left in the `put` tail after the tagged index?

## Per-policy notes

- S3-FIFO (the only policy so far):
  - Current lineage: small ratio 3–5%, baseline promotion threshold, frequency cap 3, ring-buffer ghost, tagged index with split ghost map, Fx-style hasher with moderate sizing, keyed single lookup.
  - Baseline promotion threshold, ghost size and frequency cap look near-optimal on this trace.
  - Further parameter sweeps (small ratio, hasher, cap, adaptive sizing) are saturated or harmful. Stop them.
  - Next work: promotion or ghost-policy ideas that keep main admission strict. Validate on multiple traces with repeated runs and an explicit heap measurement. Judge on hit rate and on latency changes that have a mechanism.
