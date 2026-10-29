# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v002
- allow_edits: True
- started: 2026-10-28T20:12:04+00:00

## timeline

### 20:12:13 tool: Bash

```json
{
  "command": "C=/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v002; P=/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/variants/v003; cat /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/hypotheses/H02.md; echo ===; find $C $P -type f | head -100; echo ===; cat $C/eval/fitness-vector.json; echo; cat $P/eval/fitness-vector.json; echo; cat $P/insight.md; cat $C/manifest.toml 2>/dev/null",
  "description": "Read hypothesis, fitness vectors, parent insight"
}
```

### 20:12:15 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/hypotheses/H02.md"
}
```

### 20:12:16 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v002/eval/fitness-vector.json"
}
```

### 20:12:17 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/variants/v003/eval/fitness-vector.json"
}
```

### 20:12:17 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/variants/v003/insight.md"
}
```

### 20:12:20 thinking



### 20:12:20 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v002"
}
```

### 20:12:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v002/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 20:12:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/variants/v003/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 20:12:29 thinking



### 20:12:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v002/eval"
}
```

### 20:12:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v002/manifest.toml"
}
```

### 20:12:33 thinking



### 20:12:33 chat

I've read the code and both fitness vectors. Now I'm writing the insight file.

### 20:12:43 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v002/insight.md",
  "content": "# Variant v002 Insight\n\n## Helped\n\n- hit-rate: The hypothesis predicted no change, and every trace and size matches the parent exactly. So the specialised `move_to_prot_mru` / `move_to_prob_mru` / `promote` / `demote_prot_head` routines and the MRU early-exit keep v003's eviction order exactly. The new `specialised_hit_path_matches_generic_reference` test checks the same thing directly against the retained `on_hit_generic`.\n- metadata-size: Node layout and flag encoding were left alone as instructed. Promotion writes `PROT_BIT | tail` and demotion writes a plain `tail`, which sets or clears the segment flag in the same store as the prev index. So the 24-byte node and the cache header are unchanged.\n- per-op-time (get p99, put p50): Both moved only slightly in the right direction. That fits the change: removing the `if protected` head/tail selection, the length updates on same-segment moves and the dead `set_prev(NIL)`/`set_next(NIL)` stores trims a few ALU ops and stores from the relink. It does not change the cost profile of either operation.\n\n## Hurt\n\n- per-op-time (get p50, the summary metric): The typical get got slower, which contradicts the central claim of the hypothesis. A get is dominated by the `HashMap` lookup and the dependent arena loads. The removed work was mostly predictable branches and stores to cache lines the relink already touches. Meanwhile every hit now pays an extra `idx != prot_tail` compare. Its outcome depends on the trace, so it is hard to predict, and it saves work only when the hit is already MRU, which is rare at 16K–128K capacities.\n- per-op-time (put p99): The put tail got worse. The likely cause is code layout. The fresh-key path moved into a separate `#[inline] insert_new` and still uses the generic `unlink` + `push_back`, while `on_hit` grew four `#[inline(always)]` routines that are inlined into both `get` and `put`. The larger hot functions put more pressure on the i-cache and branch predictor on miss-heavy puts, and the specialisation saves nothing on those puts.\n\n## Takeaway\n\nMaking the SLRU hit-path relinks segment-specific is safe, since policy and layout are exactly preserved, but it does not reduce per-op-time. The relink was never the bottleneck behind v003's put/get cost, so removing branches and stores from it is lost in measurement noise, and the MRU early-exit adds a branch on every hit that rarely pays off. Further per-op-time work should target the hash lookup and memory access pattern (e.g. map/arena locality or the key clone on insert), not the linked-list relinks.\n"
}
```

### 20:12:51 chat

I wrote `insight.md` in `cycles/0003/variants/v002/`. H02 kept hit rate exactly the same, but it did not make operations faster.

- **What worked:** Hit rate is identical to the parent on every trace and size, and node size is unchanged. So the new specialised move, promote and demote routines and the early return keep the parent's eviction order. A new test checks this against the old generic code path, which was kept for testing.
- **What didn't:** The main speed metric, typical get time, got slower. Put tail latency also got worse. Typical put time and get tail latency improved only slightly. Gets are dominated by the hash-map lookup, so trimming a few branches and stores from the list relink makes no measurable difference. The new "already most recent?" check adds a compare to every hit and only saves work when the key is already most recent, which is rare at these cache sizes. My reading of the put tail regression is that it comes from bigger inlined hot functions (more instruction-cache and branch pressure), but that is an inference from the code.
- **Takeaway:** The list relink was never the bottleneck. Future speed work should target the hash lookup and memory access patterns instead.

Most of the timing changes are a nanosecond or two, which may be within measurement noise. I couldn't list the `eval/` folder because it has no noise or variance data beyond `fitness-vector.json`. There was also no `manifest.toml` in the variant directory, so I worked from the overlay source, the hypothesis and the two fitness vectors.


---

- finished: 2026-10-28T20:12:52+00:00
- duration: 47.44s
- status: ok
- input_tokens: 14
- output_tokens: 3326
- cache_read_tokens: 195763
- cache_write_tokens: 32173
- cost_usd: 0.363113
