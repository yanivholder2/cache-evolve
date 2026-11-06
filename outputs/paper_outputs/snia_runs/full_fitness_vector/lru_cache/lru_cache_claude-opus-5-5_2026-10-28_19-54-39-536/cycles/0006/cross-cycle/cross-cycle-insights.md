# Cross-Cycle Insights (cycles 0001–0006)

## Recurring wins

- **Fold metadata into bits the node already has** (c1, c2, c5). This includes `u32` link indices, a segment flag in a link high bit, a counter in a padding byte, and a reprieve flag in `prev`. Policy state then costs no extra space per node. Adding a new field costs a full word, because the node has no free padding.
- **Fewer hash operations on put means a faster put** (c3–c6, four cycles running). Examples: the single-`Entry` no-clone put, cheap Fx ghost tags, and doing ghost work only on the vacant path. Put cost scales with the number of hashes per op, not with list logic. A 16-bit tag doesn't need a strong hash.
- **Representation-only changes move one axis cleanly** (c1, c4). Hit rate stays exactly the same, so a gain on the target axis is easy to attribute.
- **Second chances work best when gentle and selective** (c4–c5). A one-shot fingerprint reprieve that keeps the key in probation lifted web_0 where eager or strict admission had hurt it. It also beat simply widening probation.
- **Gate hit-rate mechanisms by cache size** (c5–c6). Enabling the reprieve only at small sizes removed the large-size hm_0 regressions.
- **Removing the victim before inserting the new key fixes the get tail** (c6). Tombstone churn from insert-before-remove was a real cause of the get-p99 rise, not noise.

## Recurring failures

- **Trying to move get p50 through list or node work** (c1, c3, c5, c6). Removing relinks, specialised relinks, CLOCK bits on protected hits, flag bit-op trimming and node compaction all left get p50 flat or made it worse.
- **Trying to move get p50 through the hash table** (c2, c6). A cheaper hasher made get slower. A custom scalar index table with fewer cache lines lost to hashbrown's SIMD probing.
- **Anything added to the hit path or the header's hot line** (c4–c6). Per-op ticks, masking `next`, extra ANDs and a larger `LruCache` header all raised get p50 sharply.
- **Per-cache side structures** (c3–c6). Ghost sets and sketches tend to hurt get p50 even when get never touches them, most likely through cache pressure.
- **Admission that is too eager or too strict** (c3). A full-key ghost that promotes straight to protected, or a miss-only tie-rejecting TinyLFU gate, both hurt web_0.
- **Tuning the probation share** (c4–c5). Online hill-climbing chased noise. A static wider share helped hm_0 but hurt web_0, so no fixed share suits both traces.
- **GCLOCK with decrement-by-one aging** (c2). The long put sweeps cost more than the narrow hit-rate gain was worth.
- **Trusting small timing or hit-rate deltas from single runs** (c5–c6). Byte-identical code swung widely between runs, and the reseeded `RandomState` moved hit rates.

## Open questions

- What actually drives the remaining get-p50 regression? Two suspects are left: helper inlining and code layout (`on_hit`/`unlink`/`push_back`), and the ghost table's cache footprint. Neither has been isolated yet.
- Does remove-before-insert eviction on stock hashbrown keep both the put gain and the tail fix?
- Is there a large-size ghost width (between capacity/8 and capacity) that recovers web_0 at 131K without readmitting hm_0's long-loop keys?
- How big is run-to-run variance? This needs a deterministic ghost seed and repeated paired parent/child runs before any delta under a few percent can be trusted.
- What does web_0 at the large size actually need? Recency segmentation, counters, strict admission and a short ghost have all failed so far.
- Untested: W-TinyLFU (a sketch that counts hits too, with a window and a cheap hash), and map/arena locality as a latency lever.

## Per-policy notes

- **LRU (baseline)**: its main contribution has been the `u32`-index layout, which carries no policy and costs nothing. All later work builds on it.
- **SLRU**: the strongest hit-rate base. It gains on every trace and size when on a compact flag-packed node. Its cost sits in put (double relink on promote/demote, per-segment branching), not in get. Lineage of the current base: c2 v003, then c4 v002 (no-clone put), then c5 v001 (size-gated `prev`-bit reprieve), then c6 v001 (Fx tags, vacant-only ghost).
- **SLRU + ghost/reprieve**: the only history mechanism that has helped web_0. It works when gentle (stays in probation), size-gated and cheaply hashed. A ghost that is too small acts as a reuse-distance filter that is too short.
- **CLOCK**: it removes relinks and shrinks nodes, but get p50 barely moves. Put tail latency from sweeps is unbounded. Its hit-rate gain is size- and trace-dependent. A CLOCK bit in the protected segment of SLRU did not help.
- **GCLOCK**: modest hm_0 gain at large size only, and slow put sweeps. Only worth revisiting if aging moves off the eviction path, e.g. bulk halving.
- **TinyLFU (bare gate)**: failed as a miss-only, tie-rejecting gate. The full W-TinyLFU variant has not been tried.
