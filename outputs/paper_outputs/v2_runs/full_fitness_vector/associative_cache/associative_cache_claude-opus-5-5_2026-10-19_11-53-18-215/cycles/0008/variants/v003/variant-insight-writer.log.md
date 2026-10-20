# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v003
- allow_edits: True
- started: 2026-10-20T09:41:51+00:00

## timeline

### 09:42:09 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles; cat 0008/hypotheses/H03.md; echo ---; cat 0008/variants/v003/eval/fitness-vector.json; echo ---; cat 0007/variants/v001/eval/fitness-vector.json; echo ---; cat 0007/variants/v001/insight.md; echo ---; ls -R 0008/variants/v003 0007/variants/v001/attempts/01/overlay; cat 0008/variants/v003/manifest.toml 2>/dev/null"
}
```

### 09:42:11 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/hypotheses/H03.md"
}
```

### 09:42:12 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v003/eval/fitness-vector.json"
}
```

### 09:42:13 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/variants/v001/eval/fitness-vector.json"
}
```

### 09:42:13 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/variants/v001/insight.md"
}
```

### 09:42:14 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v003/manifest.toml"
}
```

### 09:42:16 chat

Let me look at the overlay source.

### 09:42:17 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v003"
}
```

### 09:42:20 tool: Grep

```json
{
  "pattern": "fn pair_put_tagged",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v003/attempts/01/overlay",
  "-n": "true"
}
```

### 09:42:20 tool: Grep

```json
{
  "pattern": "fn pair_put_tagged",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/variants/v001/attempts/01/overlay",
  "-n": "true"
}
```

### 09:42:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v003/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 09:42:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/variants/v001/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 09:42:31 thinking



### 09:42:40 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v003/insight.md",
  "content": "# Variant v003 Insight\n\n## Helped\n\n- hit-rate: Cold keys (ghost misses) now take the evicted victim's slot at probation LRU instead of being rotated to `len0 / 2`. A one-hit wonder is therefore the next victim. The protected item that was just demoted to probation MRU, and keys re-admitted on a ghost hit, now survive at least one more cold miss before they are evicted. This acts as LIP/BIP-style two-strike admission without shrinking or displacing protected. The gain is modest, which fits what the code shows. When probation has 1 slot, `mid = len0 / 2 = 0`, so v001's rotate was already a no-op and the new code behaves exactly like the old. The behaviour only differs when probation has ≥2 slots: the floor-of-2 buckets of 16+ slots, or a probation that is temporarily wider after demotions.\n- per-op-time (put): The full-bucket cold path is now just two `mem::replace` calls plus `record_ghost`. The `rotate_left(1)` over `items[0..=mid]` (16-byte items) and over `tags[0..=mid]` is gone. The drop is larger at the tail than at the median, which fits a saving that only appears when probation is wide enough for `mid > 0`. While the bucket is filling, the extra cost of `insert(0)` compared with `insert(mid)` is too small to matter.\n\n## Hurt\n\nNone. metadata-size is unchanged because `Item` and the cache header were not touched. per-op-time (get) is flat, as expected: the lookup and hit paths (`find_tagged`, the protected/promotion rotates) were left alone.\n\n## Takeaway\n\nWhen probation is only 1–2 slots, where a cold key enters is still a real hit-rate lever. It works because it decides whether the newcomer or the just-demoted protected item gets evicted next. Placing history-less keys at LRU improves both hit rate and put cost at the same time, since it removes work rather than adding it. This is the first hit-rate gain on this lineage that leaves the protected cap alone. Admission policy for cold keys, not further cap tuning, is the axis that still has room. But with a 1-slot probation, LRU entry and middle entry are the same thing, so this lever is limited by how wide probation is.\n"
}
```

### 09:42:48 chat

I wrote the Insight for v003. Nothing got worse: hit rate went up a little, put got faster, and get latency and metadata size held.

- **Hit rate (helped):** v003 changes where cold keys (keys not in the ghost list) go. They now take the evicted victim's slot at the bottom (LRU end) of probation instead of the middle. A key seen only once is therefore the next one evicted. The item just demoted from protected and keys re-admitted on a ghost hit now outlast it. This is the first hit-rate gain on this lineage that doesn't touch the protected cap.
- **Put time (helped):** the full-bucket cold path no longer runs the rotate over items and tags. It is now two `mem::replace` calls plus recording the ghost. The tail dropped more than the median, which fits a saving that only shows up when probation has 2+ slots.
- **Why the gain is small:** with a 1-slot probation, the parent's middle position was already index 0, so the two variants behave the same there. The change only matters when probation has 2+ slots: buckets of 16+ slots, or a probation briefly widened by demotions.
- **Takeaway:** changing where cold keys enter is a lever that both raises hit rate and makes put cheaper. Cold-key admission looks like a better direction than more cap tuning, but how much it can give depends on how wide probation is.

There was no `manifest.toml` in the variant directory, so I worked from the hypothesis, the bucket source diff and the two fitness vectors.


---

- finished: 2026-10-20T09:42:50+00:00
- duration: 59.12s
- status: ok
- input_tokens: 14
- output_tokens: 3240
- cache_read_tokens: 172392
- cache_write_tokens: 35692
- cost_usd: 0.38487
