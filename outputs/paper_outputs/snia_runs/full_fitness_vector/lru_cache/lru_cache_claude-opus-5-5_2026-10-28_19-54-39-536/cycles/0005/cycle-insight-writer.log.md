# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0005
- allow_edits: True
- started: 2026-10-28T20:29:03+00:00

## timeline

### 20:29:07 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0005; for f in variants/*/insight.md hypotheses/*.md; do echo \"=== $f\"; cat \"$f\"; done",
  "description": "Read variant insights and hypotheses"
}
```

### 20:29:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0005/variants/v001/insight.md"
}
```

### 20:29:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0005/variants/v002/insight.md"
}
```

### 20:29:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0005/variants/v003/insight.md"
}
```

### 20:29:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0005/hypotheses/H01.md"
}
```

### 20:29:11 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0005/hypotheses/H02.md"
}
```

### 20:29:12 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0005/hypotheses/H03.md"
}
```

### 20:29:22 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0005/cycle-insight.md",
  "content": "# Cycle 0005 Insight\n\n## Cohort outcome\n\nAll three variants branched from cycle-0004 v002 (single-`Entry`, no-clone SLRU). v001 ported the fingerprint reprieve with its flag in `prev` and the ghost enabled only for small caches. v002 replaced protected-hit relinks with a CLOCK reference bit. v003 used a static 1/3 probation share at small sizes. v001 is the only Winner. It lifted both traces at 16K and matched the parent at 131K, but gave back v002's put gain by doubling hash work, and get p50 still regressed.\n\n## Patterns\n\n- Gating by size works for hit-rate mechanisms. Both v001 and v003 left 131K untouched by design, so every hit-rate change came from 16K alone. v001's gate removed the large-size hm_0 loss from cycle 4.\n- At 16K the reprieve beats resizing. A static wider probation helped hm_0 but hurt web_0, so no fixed share suits both traces. The reprieve gives extra probation time only to keys that come back, and it helped both. Selective second chances do better than a uniformly bigger window.\n- Put cost still tracks hash count. v001's separate `hash_one` for ghost tags, on top of the map's own hashing, erased the single-probe gain. This is the third cycle confirming the rule.\n- Hit-path changes still hurt get p50, even tiny ones. An extra AND in `set_protected` (v001) and CLOCK moving list work into promotion sweeps (v002) both regressed it. Removing relinks from protected hits did not lower the median. That is more evidence that get p50 is not driven by list work.\n- The timing benchmark is too noisy to rank small per-op changes. v003 changed nothing on the hot path, yet its per-op times swung widely in opposite directions. Get p99 swings in every variant now look like noise, including cycle 4's p99 rise.\n\n## Open questions\n\n- Can v001's ghost reuse the hash the map already computes (raw-entry or `hashbrown` API), and run only on the vacant path, to restore v002's put cost?\n- If `set_protected(true)` is kept to a single OR and `SC_BIT` is cleared at reprieve or eviction time, does get p50 recover?\n- How much timing movement is just run-to-run variance? Repeated paired runs of parent and child are needed before trusting per-op deltas under a few percent.\n\n## Continuation\n\nMake v001 (size-gated `prev`-bit reprieve) the base, and fix its costs next. Derive the ghost tag from the map's own hash, skip ghost work on occupied-key puts, and make `set_protected` a single OR. Drop protected-segment CLOCK and static probation-share retuning. Judge latency claims only from repeated paired runs.\n"
}
```

### 20:29:28 chat

I wrote `cycle-insight.md` in the cycle 0005 output dir. v001 is the only Winner, and the insight recommends it as the next base.

- **Cohort:** all three variants came from cycle-0004 v002.
  - v001 moved the reprieve flag into `prev` and turned the ghost on only for small caches.
  - v002 replaced protected-hit relinks with a CLOCK reference bit.
  - v003 set a fixed 1/3 probation share for small caches.
- **What v001 got:** both traces gained hit rate at 16K, and 131K matched the parent, so cycle 4's large-size hm_0 loss is gone. But it hashes keys twice on put, which cancels the parent's put gain, and get p50 still got worse.
- **Patterns:**
  - Switching mechanisms on only for small caches works.
  - At 16K, giving returning keys a second chance beats making probation bigger. The bigger window helped hm_0 but hurt web_0.
  - For the third cycle running, put cost follows the number of hash operations.
  - Even a one-instruction change on the hit path raised get p50.
  - The timing benchmark is too noisy to rank small speed changes. v003 didn't touch the hot path, yet its timings swung widely in both directions.
- **Continuation:**
  - Build on v001. Compute the ghost tag from the hash the map already computes, skip ghost work when the key is already in the cache, and make `set_protected` a single OR.
  - Drop protected-segment CLOCK and retuning the probation share.
  - Trust latency changes only from repeated runs of parent and child side by side.


---

- finished: 2026-10-28T20:29:29+00:00
- duration: 25.64s
- status: ok
- input_tokens: 8
- output_tokens: 2348
- cache_read_tokens: 89847
- cache_write_tokens: 15755
- cost_usd: 0.191001
