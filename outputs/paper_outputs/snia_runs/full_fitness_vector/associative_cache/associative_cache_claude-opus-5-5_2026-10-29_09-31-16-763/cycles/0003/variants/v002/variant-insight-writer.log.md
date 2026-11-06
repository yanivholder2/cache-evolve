# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0003/variants/v002
- allow_edits: True
- started: 2026-10-29T10:01:42+00:00

## timeline

### 10:01:47 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763; cat cycles/0003/hypotheses/H02.md; echo ----; cat cycles/0003/variants/v002/eval/fitness-vector.json; echo ----; cat cycles/0002/variants/v001/eval/fitness-vector.json; echo ----; cat cycles/0002/variants/v001/insight.md; echo ----; ls -R cycles/0003/variants/v002 cycles/0002/variants/v001 | head -80",
  "description": "Read hypothesis, fitness vectors, parent insight"
}
```

### 10:01:50 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0003/hypotheses/H02.md"
}
```

### 10:01:50 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0003/variants/v002/eval/fitness-vector.json"
}
```

### 10:01:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0002/variants/v001/eval/fitness-vector.json"
}
```

### 10:01:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0002/variants/v001/insight.md"
}
```

### 10:01:55 thinking



### 10:01:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0003/variants/v002/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 10:02:00 thinking



### 10:02:09 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0003/variants/v002/insight.md",
  "content": "# Variant v002 Insight\n\n## Helped\n\n- hit-rate: web_0 on 1024-wide buckets got back part of the scan resistance BIP had cost it. One-hit scan keys almost never come back inside the 16-eviction ghost window, so they now always land at quarter depth. No dead item is blindly promoted to the MRU end any more, where it would sit for the whole length of a long bucket. web_0 on 128-wide buckets barely moved.\n\n## Hurt\n\n- hit-rate: hm_0 on 128-wide buckets gave back most of the gain v001 got from BIP. hm_0 on 1024-wide buckets stayed flat, so the overall summary slipped below the parent's. A ghost ring holds only the last 16 evicted keys. A key gets MRU admission only if it returns within those 16 evictions, which is a very short reuse distance. hm_0's shifting working set mostly comes back later than that. As a result, ghost hits are rare, and the bucket acts almost like pure quarter-depth LIP. That is the setup that failed hm_0 on small buckets before BIP.\n- per-op-time: Put latency rose again at p50 and at p99. The p99 rise was larger. Every put miss now does a linear scan of 16 `Option<K>` slots for the ghost lookup. After an eviction it also writes a ring slot and updates the index with a modulo. All of this sits on top of the `Vec::insert(len/4, ..)` memmove that still dominates. Get is unchanged, as expected, because the ghost is never touched on the hit path.\n- metadata-size: The measured dimension did not move, because it reports `Item` and the top-level cache handle. But each bucket now carries an inline 16-slot ghost array, which is roughly 256 bytes for `u64` keys. This is real per-bucket memory that the current metric does not see.\n\n## Takeaway\n\nGhost-hit admission keeps scan resistance on large buckets, but a fixed 16-entry ghost is too short to catch hm_0's reuse. On 128-wide buckets, the signal almost never fires, so the hm_0 gain BIP had bought is lost. That gain needs a ghost window sized to the bucket (for example, a fraction of `max_size`) or a cheaper reuse filter, such as a small hashed bit set. Either would see further back for less memory and lookup cost than a linearly scanned key ring, which is now another cost on every put miss.\n"
}
```

### 10:02:16 chat

I wrote `insight.md` for v002. Replacing v001's blind every-32nd-miss promotion with a ghost list (a 16-slot ring of recently evicted keys per bucket) traded one workload for another and was slightly worse overall.

- **Helped – hit-rate:** web_0 on 1024-wide buckets got back part of the scan resistance it lost under v001. Scan keys almost never return within 16 evictions, so they now always go in at quarter depth instead of sometimes being promoted.
- **Hurt – hit-rate:** hm_0 on 128-wide buckets lost most of the gain v001 had made. A 16-key ghost is too short to catch hm_0's reuse, so promotion rarely triggers and the bucket behaves almost like plain quarter-depth insertion (LIP) again. The overall hit-rate summary ended just below the parent's.
- **Hurt – per-op-time:** put got slower at both median and p99, with the bigger rise at p99. Every put miss now scans the 16 ghost slots, and every eviction writes a ring slot. That comes on top of the mid-bucket insert, which is still the main cost. Get didn't change, since the ghost is never touched on a hit.
- **metadata-size:** the measured number didn't change, because it only counts `Item` and the top-level cache struct. Each bucket now carries about 256 bytes of ghost array that the metric doesn't see.

The takeaway proposes two follow-ups: size the ghost window relative to the bucket instead of fixing it at 16, or use a cheaper reuse filter such as a small hashed bit set.

As instructed, I left out the raw fitness numbers; they're in `eval/fitness-vector.json`.


---

- finished: 2026-10-29T10:02:17+00:00
- duration: 34.48s
- status: ok
- input_tokens: 10
- output_tokens: 2743
- cache_read_tokens: 106099
- cache_write_tokens: 12865
- cost_usd: 0.17904
