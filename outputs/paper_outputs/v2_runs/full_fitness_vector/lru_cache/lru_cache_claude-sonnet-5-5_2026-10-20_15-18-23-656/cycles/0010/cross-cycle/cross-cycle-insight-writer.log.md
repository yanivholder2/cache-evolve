# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:49:46+00:00

## timeline

### 19:49:59 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Hit-rate and speed/size are independent axes. Hit-rate comes only from policy. Speed and size come only from mechanics (u32 links, lighter hasher, head-touch, key move). This held early on the plain LRU.\n- Segmented LRU's scan resistance is established (cycle 1). Ghost-gated cold-key insertion at the LRU end of probation was the one later policy gain (cycles 7-8).\n- Trimming ghost overhead (smaller table, fewer hashes, ghost lookup only on a full cache) cut put tail latency without losing hit-rate (cycle 8).\n- Removing the hash avalanche fold on the segmented policy sped up put and shrank the struct (cycle 4). Mixing was a real cost there.\n- Bounding the second-chance eviction walk with a cap cut put p99. It traded away some hit-rate (cycle 5).\n- Larger protected share gave small, diminishing hit-rate gains (cycles 5-8).\n\n## Recurring failures\n\n- Work-shifting does not remove cost. Lazy demotion, a reference bit instead of relink-on-get, and the eviction walk each moved promotion/demotion work between operations (cycles 3-4).\n- Ghost-hit promotion fails: sending ghost hits to protected, or to probation MRU, lowered hit-rate (cycles 7, 9). The ghost filter works by making suspicious keys cheap to evict.\n- Adding bookkeeping to the put path (ghost counters, admission gates, adaptive caps, reference bits) raised put cost and state with no verified gain (cycle 10).\n- Micro-optimizations are invisible: key-clone removal, node-slot reuse, map-call fusion, further ghost shrinking, head-adjacent relink skips (cycles 4, 7, 9, 10). Map calls and allocation are not the put bottleneck.\n- Constant and position tweaks are exhausted: protected-share steps, demotion position, `put`-update promotion.\n- Winner status is unreliable for small changes. Per-op-time deltas appear on untouched paths in every cycle since 5, so they are noise. Many recent Winners likely reflect noise tolerance, not merit.\n- On a segmented policy, u32 links and bit packing gave size savings but no speed (cycle 3).\n\n## Open questions\n\n- What is the per-op-time noise floor? This has been pending since cycle 5. A repeat-run measure is still missing.\n- Does the ghost-gated design hold on non-scan workloads? This check has been pending for many cycles.\n- Where does put time actually go (hashing, list-pointer cache misses, eviction walk)? A profile is needed before choosing a lever.\n- How much heap-allocated ghost memory is there? The per-node size metric cannot see it.\n- Is there hit-rate headroom left on this trace, or is the baseline near its ceiling? Is there a lever outside insertion position and segment share?\n- Does segment bit packing or implicit segment storage help once the other pieces are in place?\n\n## Per-policy notes\n\n- Plain LRU: u32 links, a lighter hasher and a key move were pure speed and size wins (cycle 2). It has no hit-rate lever.\n- Segmented LRU: it gains hit-rate on scans but pays for promotion/demotion relinking on the hot path. Gains here are mostly size or hit-rate, rarely speed.\n- Segmented LRU with ghost-gated insertion (the cycle-8 baseline): small ghost table, single-hash gated eviction path, larger protected share. It is the current parent and is near saturation on this trace.\n- Second-chance (CLOCK-style) variants: they give a small scan-resistance gain, but get-side savings return as put and eviction cost. A cap makes it a latency/hit-rate dial.\n"
}
```

### 19:50:06 chat

I wrote `cross-cycle-insights.md` to the output dir, with the four required sections in the fixed order.

The file was generated from the cycle 1–10 ledger digest alone. I didn't read the previous insights file or any code, and it contains no invented numbers.

The main conclusions it records:
- **Wins:**
  - Hit-rate and speed/size are independent axes.
  - Segmentation's scan resistance is established, and ghost-gated cold-key insertion was the one later policy gain.
  - Trimming ghost overhead cut put tail latency.
- **Failures:**
  - Moving work between operations doesn't remove it.
  - Ghost-hit promotion and extra put-path bookkeeping don't pay off.
  - Micro-optimizations are invisible.
  - Many recent Winners likely reflect noise tolerance, not real gains.
- **Open questions:** the per-op-time noise floor, non-scan workloads, where put time actually goes, and heap-inclusive ghost memory.
- **Per-policy notes:** plain LRU, segmented LRU, the cycle-8 ghost-gated baseline (the current parent), and second-chance variants.


---

- finished: 2026-10-20T19:50:06+00:00
- duration: 19.91s
- status: ok
- input_tokens: 4
- output_tokens: 1794
- cache_read_tokens: 42177
- cache_write_tokens: 14128
- cost_usd: 0.079665
