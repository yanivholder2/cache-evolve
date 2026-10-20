# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007
- allow_edits: True
- started: 2026-10-20T19:35:44+00:00

## timeline

### 19:35:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/variants/v001/insight.md"
}
```

### 19:35:48 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/variants/v002/insight.md"
}
```

### 19:35:48 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/variants/v003/insight.md"
}
```

### 19:35:49 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/hypotheses/H01.md"
}
```

### 19:35:49 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/hypotheses/H02.md"
}
```

### 19:35:50 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/hypotheses/H03.md"
}
```

### 19:36:01 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0007/cycle-insight.md",
  "content": "# Cycle 0007 Insight\n\n## Cohort outcome\n\nTwo ghost-history variants and one micro-optimization were tried: ghost-hit keys sent straight to the protected segment (v001), cold keys inserted at the LRU end of probation unless the ghost table remembers them (v002), and an evicted-key clone removal in `put` (v003). All three were Winners, but only v002 produced a real gain, a hit-rate improvement paid for with slower put. v001 improved nothing, so its Winner status looks like noise tolerance, not merit.\n\n## Patterns\n\n- Ghost history works as a scan filter on insertion position, not as a promotion trigger. Demoting cold keys to the next-to-evict slot (v002) helped hit-rate. Promoting ghost hits into protected (v001) hurt it, because returning evictees are mostly not hot enough to displace protected entries.\n- Both ghost designs cost the same on the miss and eviction path: an extra hash, a random table access and side memory. This showed up as worse put, especially at the tail. v002 left get unchanged. v001 slowed both.\n- The ghost table lives outside `Node`, so the per-node size metric hides its memory cost. Metadata-size comparisons undercount this family of changes.\n- v003 confirms again that key-copy micro-optimizations are invisible. For a plain integer key, the clone is free, and put cost is dominated by map and list work. Its small latency shifts are within noise.\n- Authors could not read diffs, so attribution is inferred from hypotheses and fitness vectors.\n\n## Open questions\n\n- Can the ghost cost be cut, for example by hashing only on eviction and reusing the hash already computed for the map, or by a smaller or sampled table, so put p99 recovers?\n- Does a middle path work: ghost hits inserted at probation MRU, with no protected admission and no LRU-end insertion for cold keys?\n- How much of v002's gain depends on the scan-heavy trace? Does it hurt or help on non-scan workloads?\n- What is the real noise floor of per-op-time? This has been pending for several cycles.\n\n## Continuation\n\nUse v002 (ghost-gated LRU-end insertion) as the next baseline. Aim at cutting its put and memory overhead, such as a cheaper fingerprint, a smaller table, or a reused hash, and at checking hit-rate on non-scan workloads. Drop ghost-to-protected promotion (v001) and key-clone or other micro-optimizations that sit inside the noise.\n"
}
```

### 19:36:08 chat

I wrote `cycle-insight.md` in the cycle 0007 output dir. It follows the required structure and covers the cohort outcome, patterns, open questions and continuation.

All three variants were Winners, but only v002 produced a real gain. Its ghost-gated insertion at the LRU end of probation improved hit-rate, at the cost of slower put and extra side memory. v001 sent ghost hits straight to the protected segment and improved nothing, so its Winner status looks like noise tolerance. v003 removed an evicted-key clone, and its small speed shifts are within noise.

The continuation tells the next cycle to use v002 as the baseline and cut its put and memory overhead. It also says to check hit-rate on workloads other than the scan-heavy trace, and to drop ghost-to-protected promotion and clone-style micro-optimizations.


---

- finished: 2026-10-20T19:36:09+00:00
- duration: 24.23s
- status: ok
- input_tokens: 6
- output_tokens: 1822
- cache_read_tokens: 61075
- cache_write_tokens: 12131
- cost_usd: 0.073282
