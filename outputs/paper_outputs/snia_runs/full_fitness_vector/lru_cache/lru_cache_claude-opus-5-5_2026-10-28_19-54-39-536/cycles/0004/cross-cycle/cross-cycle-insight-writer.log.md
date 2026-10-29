# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-28T20:22:09+00:00

## timeline

### 20:22:26 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Representation-only changes that leave policy alone are the cleanest wins. `u32` link compaction (c1), the single-`Entry` no-clone put (c4), and specialised relinks (c3, neutral) each moved one axis, or none, without touching hit rate.\n- Packing policy state into bits the node already has keeps metadata compact. Examples are a segment flag in a link high bit and a counter in the padding byte (c2). Any new field that doesn't reuse existing space costs a full word (c1).\n- Cutting the number of hash operations is the only lever that has reliably sped up put. Every added hashed structure made put slower (c3), and removing a lookup or clone made it faster (c4).\n- SLRU's frequency/scan filter is the most consistent hit-rate gain on the MSR traces. It improved both traces at both sizes once it was on compact nodes (c2), and has been the lineage base since c2.\n- Gentle history beats aggressive history. A one-shot reprieve that keeps the entry in probation (c4) helped web_0, where an eager ghost and a strict gate had both hurt it (c3).\n\n## Recurring failures\n\n- Using the hasher or hash speed to lower get latency failed in c1 (inferred), c2, and c4. get p50 is dominated by the map probe, which looks memory-bound. A cheaper hash made it worse.\n- List or relink micro-optimisation as a speed lever: c1 and c3 both showed list work is not the get bottleneck. Gains were lost in noise.\n- Anything that touches the hit path or the header's hot cache line raises get p50 sharply. Seen with per-op ticks, masking `next`, header growth, and side structures the hit path never reads (c3, c4).\n- Aggressive admission or history state: full-key ghosts that promote straight to protected, and miss-only tie-rejecting TinyLFU gates, both starved short-reuse keys on web_0 (c3).\n- Online adaptation of the SLRU split: a hill-climber that updates every epoch chased noise and hurt hm_0 (c4).\n- GCLOCK with decrement-by-one aging on the eviction path paid long put sweeps for a narrow hit-rate gain (c2).\n\n## Open questions\n\n- web_0 at the large cache size has been unsolved since c1. Is it an admission-aggressiveness problem (c3/c4 evidence) or a probation-share problem?\n- Can the c4 second-chance bit move off `next`, for example to padding or the key/value side, so that get p50 recovers?\n- Should the reprieve and the probation share be set statically per cache size? Several small-size web_0 gains (c1, c4) suggest probation is undersized there.\n- Is the c4 no-clone put's get-p99 rise real? Would remove-before-insert remove it?\n- Side-structure get regressions: real CPU-cache pressure or benchmark interleaving? Not yet isolated.\n- Untested latency levers: table density, narrower values, hash quality/finaliser, and map/arena locality.\n- Can SLRU's promote/demote double relink on put be made cheaper?\n\n## Per-policy notes\n\n- **LRU (baseline):** policy-neutral layout work (`u32` indices, no-clone put) carries forward to any policy.\n- **SLRU:** the current lineage base (flag-packed, `u32` nodes, no-clone put). It has the best hit rate so far. Its costs are confined to put. It is weak at the small size, and web_0 at the large size is still open. Fingerprint second chance within probation is the most promising hit-rate add-on.\n- **CLOCK/GCLOCK:** a single bit cut metadata and gave mixed hit-rate gains (c1). Put tail latency from sweeps is unbounded. GCLOCK counters helped mainly hm_0 at the large size. CLOCK has been deprioritised since c2 unless aging moves off the eviction path.\n- **Ghost / A1out / TinyLFU admission:** helps only when it is tiny, fingerprint-based, and gentle. Full-key ghosts and strict gates hurt web_0 and add a hash on every put. W-TinyLFU (counting hits, window, cheap hash) is untested.\n"
}
```

### 20:22:33 chat

I wrote `cross-cycle-insights.md` in the cycle 0004 cross-cycle directory. It uses the four required sections in order and draws only on the ledger for cycles 0001–0004.

- **Recurring wins:**
  - Changes to data layout alone (narrower `u32` links, the no-clone put) made clean gains without changing hit rate.
  - Fitting policy state into bits the node already has kept nodes compact.
  - Doing fewer hash operations is the only thing that has reliably sped up put.
  - SLRU gives the most consistent hit-rate gain.
  - Gentle history mechanisms do better than aggressive ones.
- **Recurring failures:**
  - Changing the hasher to lower get latency failed in three cycles.
  - Trimming list and relink work never sped up get.
  - Any added work on the hit path, or growth of the cache header, raised get latency sharply.
  - Aggressive ghost entries and admission gates hurt web_0.
  - Adjusting the SLRU split at runtime chased noise.
  - GCLOCK paid for its narrow gain with long put sweeps.
- **Open questions:**
  - web_0 at the large cache size has been unsolved since cycle 1.
  - Can cycle 4's second-chance flag move off the `next` link?
  - Should the probation share be set statically per cache size?
  - Is the get p99 rise after the no-clone put real?
  - Are the get slowdowns from side structures real, or a benchmark artifact?
  - Table density, hash quality and memory locality are still untested as speed levers.
  - Can SLRU's put cost be cut?
- **Per-policy notes:** grouped under baseline LRU, SLRU (the current base), CLOCK/GCLOCK (set aside since cycle 2), and ghost/admission mechanisms.


---

- finished: 2026-10-28T20:22:33+00:00
- duration: 23.78s
- status: ok
- input_tokens: 4
- output_tokens: 2146
- cache_read_tokens: 37998
- cache_write_tokens: 9601
- cost_usd: 0.127344
