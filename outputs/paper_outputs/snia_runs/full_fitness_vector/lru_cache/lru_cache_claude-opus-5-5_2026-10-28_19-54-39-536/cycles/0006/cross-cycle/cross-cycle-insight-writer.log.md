# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-28T20:37:15+00:00

## timeline

### 20:37:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 0001–0006)\n\n## Recurring wins\n\n- **Fold metadata into bits the node already has** (c1, c2, c5). This includes `u32` link indices, a segment flag in a link high bit, a counter in a padding byte, and a reprieve flag in `prev`. Policy state then costs no extra space per node. Adding a new field costs a full word, because the node has no free padding.\n- **Fewer hash operations on put means a faster put** (c3–c6, four cycles running). Examples: the single-`Entry` no-clone put, cheap Fx ghost tags, and doing ghost work only on the vacant path. Put cost scales with the number of hashes per op, not with list logic. A 16-bit tag doesn't need a strong hash.\n- **Representation-only changes move one axis cleanly** (c1, c4). Hit rate stays exactly the same, so a gain on the target axis is easy to attribute.\n- **Second chances work best when gentle and selective** (c4–c5). A one-shot fingerprint reprieve that keeps the key in probation lifted web_0 where eager or strict admission had hurt it. It also beat simply widening probation.\n- **Gate hit-rate mechanisms by cache size** (c5–c6). Enabling the reprieve only at small sizes removed the large-size hm_0 regressions.\n- **Removing the victim before inserting the new key fixes the get tail** (c6). Tombstone churn from insert-before-remove was a real cause of the get-p99 rise, not noise.\n\n## Recurring failures\n\n- **Trying to move get p50 through list or node work** (c1, c3, c5, c6). Removing relinks, specialised relinks, CLOCK bits on protected hits, flag bit-op trimming and node compaction all left get p50 flat or made it worse.\n- **Trying to move get p50 through the hash table** (c2, c6). A cheaper hasher made get slower. A custom scalar index table with fewer cache lines lost to hashbrown's SIMD probing.\n- **Anything added to the hit path or the header's hot line** (c4–c6). Per-op ticks, masking `next`, extra ANDs and a larger `LruCache` header all raised get p50 sharply.\n- **Per-cache side structures** (c3–c6). Ghost sets and sketches tend to hurt get p50 even when get never touches them, most likely through cache pressure.\n- **Admission that is too eager or too strict** (c3). A full-key ghost that promotes straight to protected, or a miss-only tie-rejecting TinyLFU gate, both hurt web_0.\n- **Tuning the probation share** (c4–c5). Online hill-climbing chased noise. A static wider share helped hm_0 but hurt web_0, so no fixed share suits both traces.\n- **GCLOCK with decrement-by-one aging** (c2). The long put sweeps cost more than the narrow hit-rate gain was worth.\n- **Trusting small timing or hit-rate deltas from single runs** (c5–c6). Byte-identical code swung widely between runs, and the reseeded `RandomState` moved hit rates.\n\n## Open questions\n\n- What actually drives the remaining get-p50 regression? Two suspects are left: helper inlining and code layout (`on_hit`/`unlink`/`push_back`), and the ghost table's cache footprint. Neither has been isolated yet.\n- Does remove-before-insert eviction on stock hashbrown keep both the put gain and the tail fix?\n- Is there a large-size ghost width (between capacity/8 and capacity) that recovers web_0 at 131K without readmitting hm_0's long-loop keys?\n- How big is run-to-run variance? This needs a deterministic ghost seed and repeated paired parent/child runs before any delta under a few percent can be trusted.\n- What does web_0 at the large size actually need? Recency segmentation, counters, strict admission and a short ghost have all failed so far.\n- Untested: W-TinyLFU (a sketch that counts hits too, with a window and a cheap hash), and map/arena locality as a latency lever.\n\n## Per-policy notes\n\n- **LRU (baseline)**: its main contribution has been the `u32`-index layout, which carries no policy and costs nothing. All later work builds on it.\n- **SLRU**: the strongest hit-rate base. It gains on every trace and size when on a compact flag-packed node. Its cost sits in put (double relink on promote/demote, per-segment branching), not in get. Lineage of the current base: c2 v003, then c4 v002 (no-clone put), then c5 v001 (size-gated `prev`-bit reprieve), then c6 v001 (Fx tags, vacant-only ghost).\n- **SLRU + ghost/reprieve**: the only history mechanism that has helped web_0. It works when gentle (stays in probation), size-gated and cheaply hashed. A ghost that is too small acts as a reuse-distance filter that is too short.\n- **CLOCK**: it removes relinks and shrinks nodes, but get p50 barely moves. Put tail latency from sweeps is unbounded. Its hit-rate gain is size- and trace-dependent. A CLOCK bit in the protected segment of SLRU did not help.\n- **GCLOCK**: modest hm_0 gain at large size only, and slow put sweeps. Only worth revisiting if aging moves off the eviction path, e.g. bulk halving.\n- **TinyLFU (bare gate)**: failed as a miss-only, tie-rejecting gate. The full W-TinyLFU variant has not been tried.\n"
}
```

### 20:37:43 thinking



### 20:37:43 chat

I've written `cross-cycle-insights.md` to the cycle-0006 cross-cycle output dir. It uses the four required sections in order and draws only on the ledger for cycles 0001–0006.

**Recurring wins:**
- Putting policy state into bits the node already has avoids growing the node.
- Doing fewer hash operations per op is the only thing that has reliably made put faster, four cycles in a row.
- A second chance that keeps the key in probation is the only history mechanism that has helped web_0.
- Gating hit-rate mechanisms by cache size stopped the losses at the large size.
- Removing the victim before inserting the new key fixed the get tail.

**Recurring failures:**
- No change to list work, nodes or the hash table has improved get p50.
- Any change to the hit path makes get slower.
- Side structures such as ghosts and sketches slowed get even though get never touches them.
- Admission that is too eager or too strict hurt web_0.
- Neither online nor fixed tuning of the probation split worked.
- GCLOCK's aging cost more on put than its hit-rate gain was worth.
- Single-run results proved too noisy to rank small changes.

**Open questions:**
- What causes the remaining get-p50 regression: helper inlining and code layout, or the ghost table's cache footprint?
- Does remove-before-insert on hashbrown keep both the put gain and the tail fix?
- Is there a ghost size at 131K that helps web_0 without hurting hm_0?
- How large is run-to-run noise? Answering it needs a fixed ghost seed and repeated paired runs.
- What does web_0 at the large size actually need?
- W-TinyLFU (a sketch that also counts hits) and map/arena locality are still untried.

**Per-policy notes** cover LRU, SLRU, SLRU with the ghost reprieve, CLOCK, GCLOCK and TinyLFU, and trace how the current base evolved from cycle 0002 to cycle 0006.


---

- finished: 2026-10-28T20:37:43+00:00
- duration: 28.36s
- status: ok
- input_tokens: 4
- output_tokens: 2686
- cache_read_tokens: 40041
- cache_write_tokens: 12120
- cost_usd: 0.158704
