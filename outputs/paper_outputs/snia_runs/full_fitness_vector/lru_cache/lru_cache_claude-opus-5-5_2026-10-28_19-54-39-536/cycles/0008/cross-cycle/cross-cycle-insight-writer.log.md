# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-28T20:50:50+00:00

## timeline

### 20:51:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Compact node metadata (cycles 1–2).** `u32` link indices, with policy state packed into spare bits (a segment flag in a link's high bit, a counter in the padding byte), carry no hit-rate cost. They removed the metadata objection to frequency- and scan-aware policies. The node has no free padding, so any new field must reuse existing bits.\n- **Put cost tracks hash count (confirmed in cycles 3–7).** Removing hash operations on put helps: the single-`Entry` path with no key clone, ghost work only on the vacant path, and cheap Fx tags instead of a second SipHash. Every added hashed structure, extra `hash_one` or rehash on move gave the put gain back.\n- **Selective second chances beat uniform window changes (cycles 4, 5, 7, 8).** A gentle fingerprint-ghost reprieve that keeps keys in probation helps both traces. A static wider probation helps one trace and hurts the other, at both 16K and 131K.\n- **Size gating and ghost width as a reuse-distance filter (cycles 5–8).** Gating the reprieve by cache size removed losses at large sizes. At 131K, capacity/8 was too short, capacity/2 worked and capacity/4 is a little better. Width tuning is now nearly used up.\n- **Shrinking the ghost's byte footprint lowers get p50 (cycle 8).** This is the first lever that isolated a cause of the long-running get-p50 regression: narrower tags or fewer slots, on unchanged get code.\n- **Deterministic ghost hashing (cycles 6–7).** A zero-seeded Fx ghost made hit-rate deltas reproducible. Randomly seeded hashers had made them untrustworthy.\n\n## Recurring failures\n\n- **Pulling list or relink levers to cut get latency (cycles 1, 3, 5, 6).** Dropping links, specialised relinks, CLOCK bits on protected hits and bit-op trimming in `on_hit` never moved get p50. List work is not what drives a hit's cost.\n- **Changing the hash map itself (cycles 1, 2, 6).** A cheaper hasher made get worse. A custom scalar index table lost to hashbrown's SIMD probing.\n- **Any extra work or header growth on the hit path (cycles 4–7).** A per-op tick, masking of `next`, forced `#[inline(always)]` and a header larger than one cache line all regressed get p50.\n- **Aggressive history admission (cycle 3).** A full-key ghost that promotes straight to protected, and a miss-only, tie-rejecting TinyLFU gate, both hurt web_0. The ghost was too eager and the gate too strict.\n- **Retuning the SLRU split (cycles 4, 5, 8).** An online hill-climber chased noise. A static per-size probation share helps one trace at the other's expense.\n- **GCLOCK with decrement-by-one aging (cycle 2).** It lengthened put sweeps and gained hit rate mainly on hm_0 at the large size.\n- **Trading the get median for the tail (cycles 6–7).** Remove-before-insert and cold/hot code splits improved get p99 but lost get p50, which is the summary metric. When the remove-before-insert used a second hash, it also undid the put saving.\n- **Trusting small per-op deltas.** Byte-identical hot paths (cycles 5, 6, 8) showed timing swings of the same size as the claimed effects. Get-p99 movements in particular fit noise.\n\n## Open questions\n\n- Do capacity/4 slots and 8-bit tags stack, and does get p50 keep falling as the ghost shrinks further (4-bit tags, a bitset, a Bloom-style ghost)?\n- Do spurious reprieves from tag collisions actually happen in the hit-rate harness? A counter is needed before calling narrow tags hit-rate-neutral.\n- At 131K, can how reprieves are granted be changed (stricter conditions, a one-shot limit, recency-gated reprieves), as opposed to window size, to cut hm_0's loop-key leakage while keeping web_0's gain?\n- Should `metadata-size` count the heap ghost table? It is a large hidden cost at 131K.\n- Can eviction order be changed through a hash-preserving path (`HashTable`/raw entry) to get the tail fix without a second hash?\n- Which per-op deltas survive repeated paired parent/child runs? This must be settled before any latency claim of a few percent is trusted.\n- What else does web_0 at large sizes need beyond reprieves (W-TinyLFU with hit counting and a cheap hash, adaptive history)? It is still only partly solved.\n\n## Per-policy notes\n\n- **SLRU (lineage base since cycle 2).** It gives the best and broadest hit-rate gains across traces and sizes. Its costs sit in put (promote/demote relinks, per-segment branching), not get. The current lineage is flag-packed SLRU on `u32` nodes, a single-`Entry` no-clone put, and a size-gated `prev`-bit fingerprint reprieve with a narrow Fx-tagged ghost.\n- **CLOCK / GCLOCK.** Lazy promotion shrinks metadata but leaves get p50 flat. Its put tail grows with sweep length. GCLOCK's counter fits in padding, but decrement-by-one aging is costly and its gains are narrow. CLOCK bits in SLRU's protected segment brought no benefit.\n- **Ghost / reprieve (A1out-style history).** Only gentle, probation-only reprieves have helped. The ghost should be small, size-gated, deterministically hashed and computed only on vacant puts. Its byte footprint affects get latency even though get never reads it.\n- **TinyLFU admission.** Only the bare miss-only gate has been tested, and it failed. W-TinyLFU is untested.\n- **Adaptive / static split tuning.** Both forms consistently fail to beat targeted reprieves, because hm_0 and web_0 pull in opposite directions on probation size.\n"
}
```

### 20:51:13 chat

You've hit your 5-hour limit · resets 11pm (UTC)


---

- finished: 2026-10-28T20:51:13+00:00
- duration: 22.91s
- status: ok
- input_tokens: 2
- output_tokens: 2190
- cache_read_tokens: 15022
- cache_write_tokens: 12131
- cost_usd: 0.14386
