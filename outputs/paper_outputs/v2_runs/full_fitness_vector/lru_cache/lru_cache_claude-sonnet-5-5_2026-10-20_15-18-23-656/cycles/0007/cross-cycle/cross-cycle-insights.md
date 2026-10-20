## Recurring wins

- Policy changes drive hit-rate; mechanics changes (u32 links, hasher, touch fast path, key move) drive speed and size. The axes are largely independent, and mechanics changes on the plain LRU were pure wins (cycle 2).
- Segmentation (scan resistance) is established as a hit-rate win on the scan-heavy trace. It needs no more proof.
- Removing the hash avalanche fold (single multiply plus rotate) was a real put-speed and size win on the segmented policy.
- Insertion-position filters help scan resistance: ghost-gated LRU-end insertion for cold keys raised hit-rate. A second-chance reference bit gave a small gain with a cheaper get.
- Protected-share increases gave small hit-rate gains, with diminishing returns.

## Recurring failures

- Shifting promotion/demotion work between operations (lazy demotion, reference-bit eviction walks) only moves the cost and usually worsens put tails. The relink work itself is the cost.
- Narrow skips such as the head-adjacent relink skip add branch cost because few hits qualify.
- Packing or narrowing layout on the segmented policy saves size but not speed. Bundling the hash fold with layout changes obscured attribution.
- Ghost-hit promotion into protected hurt hit-rate. Returning evictees are not hot enough.
- Key-clone and constant-only micro-optimizations are invisible or within noise. Many recent "Winner" labels reflect noise tolerance, not improvement.
- Ghost tables cost put (extra hash, random access, side memory), and the per-node size metric hides that memory.
- Variant authors cannot read diffs, so attribution is inferred.

## Open questions

- What is the per-op-time noise floor? Repeat-run measurement is still pending after many cycles.
- Does any hit-rate gain hold on non-scan workloads? This has been pending since cycle 1.
- Can the ghost cost be cut (reuse the map hash, smaller or sampled table, cheaper fingerprint) without losing the hit-rate gain?
- Does a middle path work: ghost hits at probation MRU, with no protected admission?
- Can the capped eviction walk be tuned or made adaptive to hold put p99 without losing hit-rate?
- Can the segment be stored implicitly (list membership) to drop masking costs?
- Is hit-rate left in segment-boundary policy, or does the remaining gain lie in admission and sizing?

## Per-policy notes

- Plain LRU: u32 links and the lighter hasher gave speed and size wins. This is the cleanest mechanics gain.
- Segmented LRU: hit-rate is better on scans, but promotion/demotion relinking costs time, especially on put tails. Protected share is a weak lever. Demoted nodes are better kept warm than put at the eviction end.
- Segmented plus reference bit and capped walk: the cap trades put p99 directly against hit-rate.
- Segmented plus ghost history (current best baseline: ghost-gated LRU-end insertion): it filters scans through insertion position, with a hit-rate gain and a put and memory cost.
