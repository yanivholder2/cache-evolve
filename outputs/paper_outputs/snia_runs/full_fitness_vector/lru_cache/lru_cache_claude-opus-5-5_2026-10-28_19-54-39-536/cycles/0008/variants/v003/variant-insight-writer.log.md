# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/variants/v003
- allow_edits: True
- started: 2026-10-28T20:48:43+00:00

## timeline

### 20:48:49 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536; cat cycles/0008/hypotheses/H03.md; echo ----; C=cycles/0008/variants/v003; P=cycles/0004/variants/v002; ls -R $C $P | head -50; echo ----; cat $C/eval/fitness-vector.json; echo; cat $P/eval/fitness-vector.json; echo; cat $P/insight.md; cat $C/manifest.toml 2>/dev/null",
  "description": "Read hypothesis, fitness vectors, listings"
}
```

### 20:48:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/hypotheses/H03.md"
}
```

### 20:48:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/variants/v003/eval/fitness-vector.json"
}
```

### 20:48:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004/variants/v002/eval/fitness-vector.json"
}
```

### 20:48:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004/variants/v002/insight.md"
}
```

### 20:48:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/variants/v003/manifest.toml"
}
```

### 20:48:56 thinking



### 20:48:56 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/variants/v003"
}
```

### 20:48:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/variants/v003/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 20:48:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0004/variants/v002/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 20:49:04 thinking



### 20:49:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/variants/v003/insight.md",
  "content": "# Variant v003 Insight\n\n## Helped\n\n- hit-rate (hm_0 @ 131K): Shrinking protected from 80% to about 2/3 helped hm_0, against the hypothesis's expectation. The extra probation time let more of hm_0's second-touch keys get their promoting hit before being evicted. The smaller protected segment did not expose its loop-resident working set to eviction. So at this size hm_0 was limited by its probation window, not by protected space.\n\n## Hurt\n\n- hit-rate (web_0 @ 131K): web_0 lost hits, the opposite of what the rationale predicted. A wider probation window helps every key, which is not the same as a ghost that helps only returning keys. Moving about 13% of capacity out of protected evicted web_0's reused keys sooner than the longer probation stay rescued new ones. In net, web_0's 131K hits come from protected-segment depth, so the ghost result does not carry over to a static split.\n- hit-rate (summary): The web_0 loss is slightly larger than the hm_0 gain, so the change is roughly neutral on the summary and comes out slightly negative. The 16K points are unchanged, which confirms the `LARGE_CAPACITY` gate kept small sizes identical to the parent.\n- per-op-time (get p50, put p50/p99): These rose even though `get`, `on_hit`, `unlink`, `push_back` and `put` are unchanged from the parent. The diff touches only `try_new` and the tests, and get p99 actually fell. Most likely this is run-to-run benchmark variance, not a code cost. One alternative remains: if the benchmark uses a capacity above 32768, the new split changes how often `on_hit` takes the promote-plus-demote branch. A re-run would tell these apart.\n\n## Takeaway\n\nAt 131K, moving the probation/protected split from 80/20 to about 67/33 trades hits between the two traces: hm_0 gains and web_0 loses by slightly more. So a static split is not a free substitute for ghost-guided extra probation time on web_0. The benefit the ghost gave web_0 depends on targeting returning keys, and a wider window for all keys dilutes it. Also, because the hot paths are identical, the per-op-time shift here gives a rough idea of the benchmark's noise floor. Future comparisons of per-op-time changes between variants should be read against that noise.\n"
}
```

### 20:49:20 thinking



### 20:49:20 chat

I've written `insight.md` for v003. Hypothesis H03 didn't pan out: hit rate came out roughly flat to slightly down, not up at 131K as predicted.

- **What changed:** I compared v003's source against the parent (v002). The only code change is how the protected/probation split is set when the cache is created: above 32768 entries the probation share goes from 20% to about 33%. The 16K hit rates are exactly the parent's, so small sizes were left alone as intended.
- **Hit rate at 131K:** the two traces moved the opposite way from the hypothesis's rationale. hm_0 gained a little, while web_0 lost slightly more. The insight's reading is that a wider probation window helps every new key, but it also takes protected space away from web_0's reused keys. A ghost list gave extra time only to keys that return, so a fixed split can't stand in for it.
- **Speed:** get p50 and put p50/p99 got slower and get p99 got faster, even though `get` and `put` are unchanged from the parent. Most likely this is run-to-run benchmark noise, which gives a rough idea of how big that noise is. I couldn't rule out one other cause: I couldn't look inside `eval/` to see what cache size the benchmark uses. If it's above 32768, the new split changes how often a hit triggers a promote-plus-demote. A re-run would separate the two.


---

- finished: 2026-10-28T20:49:20+00:00
- duration: 36.79s
- status: ok
- input_tokens: 12
- output_tokens: 2910
- cache_read_tokens: 147163
- cache_write_tokens: 30217
- cost_usd: 0.329417
