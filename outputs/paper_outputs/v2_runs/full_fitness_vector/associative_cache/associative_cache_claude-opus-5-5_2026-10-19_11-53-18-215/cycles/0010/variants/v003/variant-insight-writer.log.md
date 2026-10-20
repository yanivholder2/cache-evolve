# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/variants/v003
- allow_edits: True
- started: 2026-10-20T10:06:57+00:00

## timeline

### 10:07:07 tool: Bash

```json
{
  "command": "C=/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles; cat $C/0010/hypotheses/H03.md; echo ----; cat $C/0010/variants/v003/eval/fitness-vector.json; echo ----; cat $C/0009/variants/v001/eval/fitness-vector.json; echo ----; cat $C/0009/variants/v001/insight.md; echo ----; ls -R $C/0010/variants/v003 | head -50; cat $C/0010/variants/v003/manifest.toml 2>/dev/null; diff -ru $C/0009/variants/v001/attempts/01/overlay $C/0010/variants/v003/attempts/01/overlay",
  "description": "Read hypothesis, fitness vectors, parent insight, diff"
}
```

### 10:07:08 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/hypotheses/H03.md"
}
```

### 10:07:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/variants/v003/eval/fitness-vector.json"
}
```

### 10:07:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/variants/v001/eval/fitness-vector.json"
}
```

### 10:07:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/variants/v001/insight.md"
}
```

### 10:07:11 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/variants/v003/manifest.toml"
}
```

### 10:07:13 thinking



### 10:07:14 tool: Grep

```json
{
  "pattern": "prot / 2|rotate_left|fn hit",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/variants/v003/attempts/01/overlay",
  "-n": "true"
}
```

### 10:07:14 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/variants/v003/eval"
}
```

### 10:07:16 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/variants/v003/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 10:07:17 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/variants/v001/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 10:07:25 thinking



### 10:07:31 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/variants/v003/insight.md",
  "content": "# Variant v003 Insight\n\n## Helped\n\n- per-op-time (get): The median is flat, but the tail dropped sharply. That was the cost the parent's insight blamed on promotion. A probation hit now rotates only `items[pos..t]` / `tags[pos..t]`, ending at protected's midpoint, where it used to rotate `items[pos..]` all the way to protected MRU. With protected at 14/30/60 slots in the larger buckets, that roughly halves the longest memmove on the get path.\n- per-op-time (put): The tail also fell, and the median edged down. A resident-key put goes through the same `hit()`, so a put that lands on a probation item gets the same shorter promotion rotate. The cold-key and ghost-hit insert paths did not change.\n\n## Hurt\n\nNone of the dimensions got worse. hit-rate is essentially flat, a gain within noise. That is far less than the hypothesis promised. metadata-size did not move, because midpoint placement is purely positional: no counters, and `Item` stays 16 bytes.\n\n## Takeaway\n\nSplitting protected into a two-level ladder, so that once-promoted keys sit below keys with three or more hits, barely changed which keys survive on this trace. Keys with exactly two hits evidently weren't pushing hot keys out of protected, so the hit-rate thesis does not hold. What the change did do is cut the promotion rotate roughly in half, which recovered most of the get/put tail latency that the 15/16 cap had added. It costs nothing in hit rate, so it is worth keeping as a latency fix. It also suggests that reordering inside protected is a weak lever for hit rate here.\n"
}
```

### 10:07:36 thinking



### 10:07:36 chat

I've written `insight.md` for v003. Midpoint promotion did not give the hit-rate gain the hypothesis predicted. Its real effect is lower tail latency.

- **Hit rate:** essentially flat compared with the parent (cycle 0009 v001). The gain is too small to tell apart from noise. Ranking keys with three or more hits above keys with exactly two seldom changes which keys survive on this trace.
- **Get latency:** the median is unchanged and the tail fell sharply. A probation hit now moves items only as far as the middle of protected instead of all the way to its MRU end. That roughly halves the longest shift on the get path, which the parent's insight named as the cause of its tail-latency rise.
- **Put latency:** the tail also fell and the median dipped slightly. Puts to keys already in the cache go through the same hit logic, so they get the shorter move too.
- **Metadata size:** unchanged. The change only reorders items, so it adds no counters and the 16-byte `Item` stays the same.

The takeaway in the file: keep this change as a latency fix, since it costs nothing in hit rate and wins back most of the tail latency that the parent's larger protected segment added. Reordering items inside protected looks like a weak way to raise hit rate here.


---

- finished: 2026-10-20T10:07:36+00:00
- duration: 38.81s
- status: ok
- input_tokens: 12
- output_tokens: 2977
- cache_read_tokens: 152228
- cache_write_tokens: 38573
- cost_usd: 0.398618
