## Recurring wins

- A second-hit admission filter is the one real, repeatedly confirmed win (cycles 0001, 0002, 0004). It keeps one-touch scan items out of hot tiers, and its flag fits in spare bits, so the item struct stays at baseline size.
- A small, sparse, aged doorkeeper beats an in-item flag on put cost, because it skips insert and eviction work on one-hit keys. Footprint and aging drive this: one cache line, aged often (0006 to 0008).
- Metadata-free designs hold up: new state packed into spare bits (counters, ref bits, flags) never grew the item.
- Removing bookkeeping the policy never reads helped once (0004). Later deletions (0005, 0007) gave nothing, so that vein is exhausted.

## Recurring failures

- Hot-path edits to get/put (lookup atomics, early returns, free-slot fast paths, promotion skips, hoisting, tag-first scans, reader-writer locks, resident-key shortcuts) regressed or did not improve latency in seven or more consecutive cycles. Time is evidently not spent there, and the edits add branches or work.
- Richer admission or recency signals (multi-bit counters, ref bits, ghost fingerprints) added almost no hit-rate once the filter was in place, and cost latency.
- Stacking position levers on admission levers overlaps and does not add. Insert position alone is exhausted.
- Lossy, unaged filters (a small ring, a direct-mapped fingerprint table) cut latency but lose hit-rate. Oversized filters recover hit-rate but cost put latency.
- Doorkeeper tuning (aging threshold, probe count) and warm-up admission are weak levers with small trade-offs.
- Frequency-counter eviction with aging added per-hit and per-eviction work and gained no hit-rate.
- "Winner" status is weak evidence. Flat metadata plus near-flat hit-rate qualify a variant even when latency regresses.
- Regressions often appear on untouched paths, which points to layout or measurement noise. Many mechanism explanations are inferred, because authors could not read the source.
- Empty overlays (0010 v002) are implementation failures and say nothing about the idea.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? It has been asked for seven cycles and is still unmeasured. It blocks crediting any latency result.
- Where do get and put time actually go (locking, memory access, scan, allocation)? Nobody has profiled it.
- Do the conclusions hold on a second trace with scans or frequency skew? Only the LIRS trace has been used, and frequency-aware or eviction levers may matter elsewhere.
- Is the one-bit second-chance flag's latency drop (0010 v003) real? Does it stack with the aged doorkeeper?
- Are the small hit-rate gains (aged doorkeeper, second-chance flag) real or trace noise?
- Incremental doorkeeper reset remains untested.

## Per-policy notes

- DASH segment with promotion and insert-position policies (0001, 0002): small gains, with position levers overlapping.
- Second-hit filter, bit-packed (0002, 0004): the base that works. Hit-rate is saturated beyond it on this trace.
- Doorkeeper-style admission (0006 to 0009): the one-cache-line aged two-probe version (0008 v001) is the current incumbent for policy work. Further tuning is not worthwhile.
- Eviction (0010): the first eviction-side cycle. A one-bit second-chance flag looks cheap and mildly positive, but is unverified. Frequency counters are not worth it.
- Locking and bucket layout (0004 to 0006): the benchmark looks uncontended and buckets mostly full, so reader-writer locks, tags and insert fast paths have nothing to exploit.
