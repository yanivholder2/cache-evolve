# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0004/variants/v002
- allow_edits: True
- started: 2026-10-29T10:13:12+00:00

## timeline

### 10:13:17 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles; cat 0004/hypotheses/H02.md; echo ----; cat 0004/variants/v002/eval/fitness-vector.json; echo ----; cat 0002/variants/v001/eval/fitness-vector.json; echo ----; cat 0002/variants/v001/insight.md; echo ----; ls -R 0004/variants/v002 0002/variants/v001 | head -80; cat 0004/variants/v002/manifest.toml"
}
```

### 10:13:19 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0004/hypotheses/H02.md"
}
```

### 10:13:20 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0004/variants/v002/eval/fitness-vector.json"
}
```

### 10:13:20 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0002/variants/v001/eval/fitness-vector.json"
}
```

### 10:13:21 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0002/variants/v001/insight.md"
}
```

### 10:13:21 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0004/variants/v002/manifest.toml"
}
```

### 10:13:24 thinking



### 10:13:25 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0004/variants/v002"
}
```

### 10:13:25 tool: Glob

```json
{
  "pattern": "attempts/**/*",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0002/variants/v001"
}
```

### 10:13:27 thinking



### 10:13:28 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0004/variants/v002/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 10:13:29 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0002/variants/v001/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 10:13:29 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0004/variants/v002/eval"
}
```

### 10:13:42 thinking



### 10:13:51 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0004/variants/v002/insight.md",
  "content": "# Variant v002 Insight\n\n## Helped\n\n- hit-rate: web_0 on 128-wide buckets is the only cell that improved, and only a little. On a small bucket a recently evicted key can come back within the ghost window, and this is the one case where the change actually caught real reuse instead of firing BIP's blind every-32nd-miss promotion.\n\n## Hurt\n\n- hit-rate: hm_0 fell at both bucket widths and web_0 fell clearly on 1024-wide buckets, so the summary dropped below the parent's. The filter has about one bit per recorded eviction (2·W bits, reset after 2·W evictions), so it is not sparse. Late in each epoch most bits are set, and on average roughly a third of all misses test \"reused\" by false positive. That sends far more one-touch keys to the MRU end than BIP's 1-in-32 did, which removes scan resistance most on wide buckets, where a dead MRU item lingers longest. The hypothesis called false positives \"harmless,\" and that turned out to be wrong. The all-at-once reset also leaves the filter nearly empty just after each clear, so hm_0's shifting working set loses the steady trickle of MRU admissions that BIP guaranteed.\n- per-op-time: Put cost rose sharply. Each miss now runs SipHash (`DefaultHasher`) twice, once to test the incoming key and once to record the victim, and every 2·W evictions it also zeroes the whole bit set. Get rose too, even though the get path is untouched. The most likely cause is the larger bucket struct (an extra `Vec` header, mask and counter) plus one more heap allocation per bucket, which spreads bucket headers over more cache lines during lookup. Run-to-run noise may also play a part.\n\n## Takeaway\n\nA hashed ghost bit set only works as a reuse signal if it is sparse. With about one bit per tracked eviction plus a periodic full reset, it fills up and acts like a high-rate random MRU promoter, which is worse than BIP's fixed 1/32 on scan-heavy traces and less steady on shifting ones. A retry would need several bits per tracked key (or fewer tracked evictions), gradual aging instead of a full clear, and a cheap integer mix instead of `DefaultHasher`. Otherwise the per-miss hashing alone wipes out any latency budget.\n"
}
```

### 10:13:59 thinking



### 10:13:59 chat

I've written `insight.md` for v002. The change did not work: hit rate went down overall and both get and put got slower than in v001.

- **Hit rate:** Only web_0 on the 128-wide buckets improved, and only slightly. hm_0 fell at both widths and web_0 fell clearly on the 1024-wide buckets. The likely cause is that the "recently evicted" bit set isn't sparse enough. It holds about one bit per recorded eviction and clears only after 2·W evictions, so it fills up between clears. By my estimate, about a third of all misses then look like reused keys by accident and get placed at the most-recently-used end, far more than the old 1-in-32 rule. That costs the scan resistance web_0 needs. Because the whole set is cleared at once, there are also stretches right after each clear when almost nothing gets promoted, which hurts hm_0's shifting working set.
- **Put latency:** Each miss now hashes twice with the standard library's default hasher, once for the new key and once for the evicted one. Every 2·W evictions it also zeroes the whole bit set.
- **Get latency:** This rose even though the get code didn't change. My best guess is the larger bucket struct and the extra memory allocation per bucket, which spread lookups over more cache lines, but run-to-run noise could also be part of it.

The takeaway in the file: a retry would need several bits per tracked key (or fewer tracked evictions), old entries aged out gradually instead of all at once, and a cheap integer hash.

These explanations come from reading the code and doing a back-of-envelope estimate, not from new measurements. The only build-related files I had were the two variants' copies of the bucket source file; v002 had no `manifest.toml`, and my file listing commands were unavailable, so I couldn't see anything else in its eval folder.


---

- finished: 2026-10-29T10:14:00+00:00
- duration: 47.54s
- status: ok
- input_tokens: 12
- output_tokens: 4039
- cache_read_tokens: 134983
- cache_write_tokens: 17272
- cost_usd: 0.246001
