# Cross-Cycle Insights (through cycle 0007)

## Recurring wins

- **Compact metadata (cycles 1–2).** `u32` link indices, plus policy state packed into link high bits or padding bytes, shrink nodes without touching hit rate. Policy state that fits in bits the node already has costs no metadata. Adding a new field costs a full word, because the node has no padding to absorb it.
- **Put cost tracks hash count (cycles 3–7, five times).** Removing hash operations makes put faster: single-`Entry`, no-clone insert, ghost work only on the vacant path, and cheap Fx/fingerprint tags. A short tag does not need a strong hash.
- **Gentle, selective admission (cycles 4–7).** A one-shot fingerprint-ghost reprieve that keeps a returning key in probation helps both traces. It beats aggressive history and uniformly resizing the window. It is the only history mechanism that has lifted web_0.
- **Size-gating hit-rate mechanisms (cycles 5–7).** Setting each mechanism per cache size stops losses at one size from cancelling gains at the other. Ghost width acts as a reuse-distance filter, and capacity/2 at 131K gave the first clean web_0 gain at the large size.
- **Deterministic ghost hashing (cycles 6–7).** A zero-seeded Fx ghost made hit-rate deltas reproducible. Before that, a randomly seeded hasher made them untrustworthy.

## Recurring failures

- **Chasing get p50 (cycles 1–7).** Nothing has moved it: list removal, relink specialisation, bit-op trimming, a cheaper hasher, a custom index table, forced inlining, cold splits, or changing eviction order. Any extra work on the hit path or in the header's hot cache line makes it worse. Side structures make it worse even when get never touches them.
- **Trading median for tail (cycles 6–7).** Get-p99 fixes have repeatedly cost get p50, and several unrelated changes "fixed" the same tail. The likely cause is noise or general layout sensitivity, not any one mechanism.
- **Aggressive or heavy history (cycle 3).** Full-key ghosts that promote straight to protected, and miss-only, tie-rejecting sketch gates, both hurt web_0. Each also adds a full hash to put.
- **Online or static tuning of the SLRU split (cycles 4–5).** The hill-climber chased noise. No single static probation share suits both traces.
- **Hash or index speed tricks (cycles 2, 6, 7).** A cheaper hasher, a scalar tombstone-free table, and remove-before-insert using a second hash all lost to stock hashbrown on get p50 or on put.
- **Timing noise.** Single runs cannot rank per-op changes below a few percent, and byte-identical code has swung widely between runs.

## Open questions

- Where does ghost width peak at 131K? Test capacity/4, or stricter reprieves at the large size, against capacity/2's small hm_0 leak.
- Should `metadata-size` count the heap ghost table? It is a large hidden cost at 131K that the metric doesn't show.
- Can eviction order be changed through a path that keeps the hash (`HashTable`/raw entry), so it costs no extra hash?
- Does the `#[cold]` split without `#[inline(always)]` keep the tail gain without the p50 loss?
- How large is run-to-run variance? Repeated paired parent/child runs are still missing. They are needed before any latency claim can be trusted.
- What drives get p50 at all? The remaining suspects are code layout and the cache footprint of side structures.

## Per-policy notes

- **SLRU (flag-packed, `u32` nodes):** the lineage base since cycle 2. It improved hit rate on every trace at both sizes. Its costs are on put, from the promote/demote double relink. Current best: single-`Entry` put, plus a size-gated fingerprint reprieve in `prev` (Fx tags, vacant-only), plus a capacity/2 ghost at 131K.
- **CLOCK / GCLOCK:** removing relinks shrank metadata but did not lower get p50. Put tail and sweep length are the weak points, and decrement-by-one aging is too costly on the eviction path. Gains were mostly hm_0. CLOCK bits inside SLRU's protected segment did not help (cycle 5).
- **TinyLFU-style admission:** only a miss-only gate has been tried, and it starved short-reuse keys. W-TinyLFU (hits counted, a window, a cheap hash) is untested.
- **Traces:** hm_0 responds to frequency and loop filtering. web_0 needs gentle second chances for returning keys and a long enough ghost window at 131K. At 16K, probation looks undersized for web_0.
