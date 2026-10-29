# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/variants/v002
- allow_edits: True
- started: 2026-10-29T08:11:31+00:00

## timeline

### 08:11:36 tool: Bash

```json
{
  "command": "C=/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles; cd $C; cat 0010/hypotheses/H02.md; echo ----; cat 0010/variants/v002/eval/fitness-vector.json; echo ----; cat 0009/variants/v001/eval/fitness-vector.json; echo ----; cat 0009/variants/v001/insight.md; echo ----; ls -R 0010/variants/v002 0009/variants/v001 | head -50; cat 0010/variants/v002/manifest.toml",
  "description": "Read hypothesis, fitness vectors, parent insight, listings"
}
```

### 08:11:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/hypotheses/H02.md"
}
```

### 08:11:39 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/variants/v002/eval/fitness-vector.json"
}
```

### 08:11:40 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0009/variants/v001/eval/fitness-vector.json"
}
```

### 08:11:40 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0009/variants/v001/insight.md"
}
```

### 08:11:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/variants/v002/manifest.toml"
}
```

### 08:11:44 thinking



### 08:11:44 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/variants/v002"
}
```

### 08:11:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/variants/v002/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 08:11:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0009/variants/v001/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 08:11:55 thinking



### 08:12:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/variants/v002/insight.md",
  "content": "# Variant v002 Insight\n\n## Helped\n\n- hit-rate (hm_0/16K, summary): hm_0/16K gained much more than any other point and accounts for nearly all of the summary gain. At 16K the ghost is full width (`capacity.next_power_of_two()`), so it keeps entries long enough for a key to go through ghost hit, SC|RET stay, eviction without a hit, and a second ghost hit. On hm_0, keys that do this have a stable reuse distance a little above the cache size, and sending them to the protected MRU keeps them through the next probation churn, where the parent's single reprieve did not.\n- hit-rate (hm_0/131K): The gain here is marginal. At 131K the ghost is only capacity/4 wide, so it seldom remembers a key long enough for a second ghost hit, and few keys get escalated.\n- per-op-time (get p50, the summary; get p99): `on_hit` changed in only two ways: the AND immediate is now `!(SC_BIT | RET_BIT)`, and the demotion branch moved into an `#[inline] demote_overflow`. Neither adds work, so the drop most likely comes from codegen or layout changes, or from noise, not from a real speedup. Treat it as \"get was not harmed\" rather than as a gain.\n- per-op-time (put p50): The ghost table is the same size as the parent's. The new work in the `Vacant` arm is a mask, a bit read and a branch, which is too little to slow the common put.\n\n## Hurt\n\n- hit-rate (web_0/16K, web_0/131K): Both web_0 points slipped slightly. This is the opposite of what the hypothesis predicted for web_0/131K. Taking a bit for `repeat` leaves 7-bit tags, which roughly doubles collisions. A spurious match can now also pick up a stale repeat bit, which escalates an unrelated key into protected and pushes a real protected LRU down via `demote_overflow`. On web_0 there are too few true repeat returners to offset this, and at 131K the capacity/4 ghost window is too short for many to qualify.\n- per-op-time (put p99): The tail rose slightly. An escalated put now does a protected `push_back` and possibly a demotion (`unlink` + `push_back`) on top of the eviction, so the rare escalation path is longer than before.\n\n## Takeaway\n\nEscalating keys only after a second ghost hit works when the ghost is wide enough to see that second return. That holds at 16K, where hm_0 benefited a lot. At 131K, the quarter-width ghost almost never catches a second return, so the change does little on hm_0 and slightly hurts web_0, where the 7-bit tag collisions cost more than the few escalations gain. To hit the original web_0/131K target, a follow-up would need to keep the repeat signal without giving up a tag bit, or widen the ghost window above `GHOST_MAX_CAPACITY`. Changing the escalation rule alone will not get there.\n"
}
```

### 08:12:14 thinking



### 08:12:14 chat

I wrote the Insight file for v002 at `cycles/0010/variants/v002/insight.md`. v002 sends a key straight to the protected segment the second time it comes back after eviction. That raised overall hit-rate, but not where H02 predicted: almost all of the gain is on hm_0 at 16K, while web_0 at 131K got slightly worse.

- **What helped:** at 16K the ghost (the table of recently evicted keys) is full width. It remembers keys long enough to see a second return, so on hm_0 those keys now stay cached. hm_0 at 131K improved only a little, because the ghost there is a quarter of the cache size and rarely sees a second return.
- **What hurt:**
  - **web_0 at both sizes:** this dipped slightly. The flag that marks a second return takes one bit from the ghost's tag, and 7-bit tags collide about twice as often. A false match can now also move an unrelated key into protected and push a genuinely reused key out.
  - **put p99:** this went up a little, because a put that escalates a key can also trigger a demotion.
- **Latency:** get p50 fell, but the get path does no less work than the parent's. The file treats this as code-layout effects or noise, meaning get wasn't harmed, not as a real speedup.
- **Takeaway:** this change can't reach the web_0/131K target. A follow-up would need to keep the repeat flag without giving up a tag bit, or make the ghost wider at large capacities.

I couldn't list the files under `eval/`, so I worked only from the fitness vector, the source code of both variants, the parent's insight file and the hypothesis. v002 has no `manifest.toml`.


---

- finished: 2026-10-29T08:12:15+00:00
- duration: 43.42s
- status: ok
- input_tokens: 12
- output_tokens: 3329
- cache_read_tokens: 168387
- cache_write_tokens: 49752
- cost_usd: 0.498321
