## Recurring wins

- Behavior-preserving rewrites that remove a mechanism-identified cost keep winning: hit rate stays identical and the targeted latency drops. Examples: scan consolidation, hash-index membership, O(1) ring-buffer ghost, single location-tagged index.
- The `put` tail has been peeled in layers. Linear scans came first, then ghost maintenance, then multiple hash-map probes per `put`. Each fix moved `put` only when it hit the real cost.
- Only changes with a clear mechanism are credited. Latency shifts on paths a change never touched are treated as noise.
- A small queue of about 5% has given modest hit-rate gains twice, from clean baselines, and the 10% change before it also helped a little. The direction is consistent but small, and every result comes from one trace.

## Recurring failures

- Anything that loosens main admission or gives main-evicted keys special treatment has lowered hit rate. This includes recycling into the shared ghost, a freq head start on re-entry, a larger ghost bound, a looser promotion threshold, and a larger small queue. The small queue's one-hit-wonder filter matters more than quick promotion.
- Latency-only "wins" with no mechanism keep being selected: constant tweaks, the rotation cap, and the head-start ghost. Latency is not a trustworthy selection signal on its own.
- Micro-optimizations are exhausted: frequency store, small-to-main move, deque pre-sizing, a CLOCK-style rewrite of main eviction, and a cheaper hasher on its own. They land within noise and do not touch the `put` tail.
- Per-entry `metadata-size` hides real footprint growth. The index, the ring and the tagged index each grew whole-cache memory.

## Open questions

- Do the gains hold across other traces, cache sizes and repeated runs? All evidence is workload-a at one size.
- Can the tagged index's footprint and `get` p99 be reduced (ghost keys kept out of the main index, better table sizing)? Does the cheap hasher stack on it?
- What is left in the `put` tail after the tagged index?
- Is there a hit-rate idea that does not loosen main admission? Hit-rate work has barely moved since the small-ratio tweaks. An O(1) tagged variant of the ghost for main-evicted keys is the only untried form.
- Can the tests deleted during the scan refactor be restored without losing its gains?
- Is the 5% small ratio real, or noise?

## Per-policy notes

- S3-FIFO, small queue: 10% beat the baseline. 15% regressed, and 5% gave small gains. The curve is flattening, so stop sweeping it unless other traces disagree. The baseline promotion threshold looks like a local optimum.
- S3-FIFO, ghost: A shared ghost that main evictions also enter dilutes the filter. A separate ghost gave an unreliable small gain, and its variants later regressed. The ghost bound looks near its sweet spot. The ring-buffer ghost made maintenance O(1).
- S3-FIFO, main eviction: Rewrites such as the CLOCK-style pop/push and the rotation cap showed nothing to gain. The frequency counter already saturates low.
- Index and latency base: The current base is the tagged index, with the ring ghost and scan consolidation. Build further `put` and `get` work on it. Judge on hit rate and on mechanism-backed latency, with repeated runs.
