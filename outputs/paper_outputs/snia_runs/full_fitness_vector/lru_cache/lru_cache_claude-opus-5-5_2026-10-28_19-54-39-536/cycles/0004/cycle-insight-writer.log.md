# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004
- allow_edits: True
- started: 2026-10-28T20:21:40+00:00

## timeline

### 20:21:44 tool: Bash

```json
{
  "command": "for f in variants/v00*/insight.md hypotheses/H0*.md; do echo \"=== $f\"; cat \"$f\"; done",
  "description": "Read variant insights and hypotheses"
}
```

### 20:21:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004/variants/v001/insight.md"
}
```

### 20:21:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004/variants/v002/insight.md"
}
```

### 20:21:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004/variants/v003/insight.md"
}
```

### 20:21:48 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004/hypotheses/H01.md"
}
```

### 20:21:49 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004/hypotheses/H02.md"
}
```

### 20:21:49 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004/hypotheses/H03.md"
}
```

### 20:22:02 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004/cycle-insight.md",
  "content": "# Cycle 0004 Insight\n\n## Cohort outcome\n\nAll three variants branched from cycle-0002 v003 (flag-packed SLRU on `u32` nodes). v001 tried an adaptive protected/probation split, v002 a single-`Entry` put path with no key clone, and v003 a gentle fingerprint-ghost second chance inside probation. The Winners are v002 and v003. v002 is the only clean gain: put got faster with policy and layout unchanged. v003 is the first history mechanism that helped web_0 instead of hurting it, though the net hit-rate gain is small and get latency regressed.\n\n## Patterns\n\n- Cutting hash operations on put works. v002 bears out the cycle-0003 finding that put cost tracks hash count. It is the first latency lever across four cycles that moved its target without hurting hit rate or metadata.\n- Gentle admission solves web_0's sensitivity. A one-shot reprieve that stays in probation lifted web_0 at both sizes, which reverses the damage cycle 3's ghost and gate did. Its benefit is size-dependent, though: hm_0 gained at the small size and lost slightly at the large size.\n- Small per-cache hit-rate effects are hidden by noise. v001's one-epoch hill-climber kept chasing workload swings and lost on hm_0. Its only win was web_0 at the small size, which again suggests probation is undersized there.\n- Any extra work on the hit path is very costly for get. v001's per-op tick and v003's masking of `next` both raised get p50 sharply. Header growth past one cache line and heap side arrays probably add to this. Both policy variants also grew the `LruCache` header. Repeated across cycles, the rule is that the `on_hit` link path and the header's hot cache line must stay untouched.\n- v002's get p99 rose although its get code was identical. This may be noise, or the insert-before-remove order may be changing the hash table's internal state. It is unresolved.\n\n## Open questions\n\n- Is v002's get-p99 rise real? If so, would removing before inserting (so the new key reuses the freed slot) keep the put gain without the tail?\n- Can v003's second-chance bit live somewhere that is free on the hit path, such as a padding byte or the key/value side rather than `next`, so get p50 recovers?\n- Should the reprieve be gated by cache size, applying only when probation is small relative to the working set, so it stops costing hm_0 at the large size?\n- Would a static per-size retune of the probation share capture v001's web_0 small-size gain with no runtime cost?\n\n## Continuation\n\nUse v002's single-`Entry`, no-clone put as the new default base. Re-check its get p99, and if it is real, try remove-before-insert. If you pursue hit rate, port v003's probation-only reprieve onto that base, with the flag stored off the `next` link and the header kept within one cache line, and consider scaling it by cache size. Drop online hill-climbing of the SLRU split. Try a static per-size probation share instead.\n"
}
```

### 20:22:09 thinking



### 20:22:09 chat

I wrote the Cycle 0004 insight to `cycles/0004/cycle-insight.md`. All three variants started from cycle-0002 v003, the SLRU cache with flags packed into compact nodes.

- **v002 (Winner):** the only clear gain. It rewrote `put` to do one map lookup instead of two, and it moves the evicted key out of its node instead of copying it. That made `put` faster, with hit rate and node size exactly unchanged. One loose end: `get` p99 went up even though the `get` code didn't change. It could be noise or a side effect of inserting the new key before removing the old one.
- **v003 (Winner):** the first history mechanism that helped web_0 instead of hurting it. A key recently evicted from probation gets one extra round in probation. The overall hit-rate gain was small: hm_0 improved at the small size and slipped a little at the large size. It also made `get` p50 sharply slower, because the extra flag lives in the `next` link that every hit updates.
- **v001 (not a Winner):** the adaptive protected/probation split kept chasing noise from one epoch to the next. It lost hit rate on hm_0, and its per-operation bookkeeping made every operation much slower. Its only gain was web_0 at the small size.

What the file tells the next cycle:
1. Use v002's `put` path as the new base and re-check its `get` p99. If the rise is real, try removing the old key before inserting the new one.
2. Move v003's second-chance bit onto that base, store it somewhere hits never touch, and keep the cache header within one cache line.
3. Drop online tuning of the split and try a fixed probation share chosen per cache size instead.


---

- finished: 2026-10-28T20:22:09+00:00
- duration: 28.76s
- status: ok
- input_tokens: 8
- output_tokens: 2446
- cache_read_tokens: 89895
- cache_write_tokens: 16264
- cost_usd: 0.197043
