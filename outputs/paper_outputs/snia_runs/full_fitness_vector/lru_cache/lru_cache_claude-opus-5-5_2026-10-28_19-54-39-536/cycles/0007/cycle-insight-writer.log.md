# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007
- allow_edits: True
- started: 2026-10-28T20:43:54+00:00

## timeline

### 20:43:59 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007; for f in variants/v00*/insight.md hypotheses/H0*.md; do echo \"=== $f\"; cat \"$f\"; done",
  "description": "Read variant insights and hypotheses"
}
```

### 20:44:01 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/variants/v001/insight.md"
}
```

### 20:44:02 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/variants/v002/insight.md"
}
```

### 20:44:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/variants/v003/insight.md"
}
```

### 20:44:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/hypotheses/H01.md"
}
```

### 20:44:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/hypotheses/H02.md"
}
```

### 20:44:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/hypotheses/H03.md"
}
```

### 20:44:09 thinking



### 20:44:20 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0007/cycle-insight.md",
  "content": "# Cycle 0007 Insight\n\n## Cohort outcome\n\nTwo variants targeted latency on the cycle-0004 v002 base without changing policy. v001 split eviction and promotion into cold functions and forced the hot helpers inline. v003 removed the victim before inserting the new key on stock hashbrown. v002 turned the cycle-0006 v001 reprieve on at 131K with a capacity/2 ghost. The Winners are v001 and v002. v002 is the clean gain: web_0 at 131K improved more than hm_0 dipped, and 16K was unchanged. v001 won only on the get tail, and its get p50 got worse.\n\n## Patterns\n\n- Two unrelated changes both \"fixed\" cycle 4's get-p99 rise. One was a pure layout split (v001), the other a change in eviction order (v003). Both also made get p50 worse. When different causes produce the same effect, noise or broad layout sensitivity is a more likely explanation than any one mechanism. Neither claimed cause has been established.\n- Get p50 still resists every lever. Forced inlining made it worse, and v003 changed no get-path instruction and it still rose. Under a p50 summary, trading the median for the tail has now lost in two variants.\n- Put cost tracks hash count, now for the fifth cycle. v003's extra SipHash on every evicting put undid the single-lookup saving.\n- The ghost window works as a reuse-distance filter. Cycle 6 showed capacity/8 is too short for web_0 at 131K. capacity/2 catches most of web_0's useful re-references and lets in only a few of hm_0's loop keys. The best width probably lies between the two. With the zero-seeded Fx ghost, hit-rate deltas can now be trusted.\n- `metadata-size` does not count the heap ghost table. At 131K the ghost is a large hidden memory cost.\n- Two of the three variant insights were written without access to the overlay source. Their explanations assume the hypothesis was implemented as written.\n\n## Open questions\n\n- Is capacity/4 (or stricter reprieves at large sizes) a better point on the web_0-gain/hm_0-leak trade-off than capacity/2?\n- Does v001's `#[cold]` split without the `#[inline(always)]` hints keep the tail fix without the get-p50 loss?\n- Is any of this cycle's get-p50 or get-p99 movement real? None of it has been confirmed by repeated paired runs.\n\n## Continuation\n\nUse cycle-0007 v002 (capacity/2 ghost at 131K) as the new base. Tune the large-size ghost width toward capacity/4 or tighten large-size reprieves. Consider counting the ghost table in metadata. Drop forced `#[inline(always)]` and drop remove-before-insert using a second hash. Retry an eviction-order change only through a hash-preserving path (`HashTable`/raw entry). Stop chasing get p99 or p50 until paired repeated runs show the effect is larger than noise.\n"
}
```

### 20:44:25 chat

I've written the Cycle 0007 insight to `cycle-insight.md` in the output dir.

- **Cohort outcome:** v002 is the only clean gain. Turning the ghost on at 131K at capacity/2 helped web_0 more than it hurt hm_0, and 16K results stayed exactly the same. v001 was a Winner only on get p99, and its get p50 got worse. v003 lost: removing the victim before inserting added a hash to every evicting put, and get p50 also rose.
- **Main pattern:** v001 changed only code layout and v003 changed only the eviction order. Both fixed cycle 4's get-p99 rise, and both made get p50 worse. Two unrelated changes with the same effect point to noise or broad layout sensitivity, not to either claimed cause. Get p50 has now ignored every lever tried. Put cost tracks hash count for the fifth cycle in a row.
- **Caveats recorded:** `metadata-size` doesn't count the 128 KB ghost table at 131K, so that memory cost is hidden. Two of the three variant insights were written without access to the source code. Their explanations assume each hypothesis was implemented as written.
- **Continuation:** Use cycle-0007 v002 as the new base. Tune the large-size ghost toward capacity/4, or make reprieves stricter at large sizes. Drop forced inlining and remove-before-insert that costs a second hash. Retry the eviction-order change only through a path that reuses the existing hash. Stop chasing get latency until repeated paired runs show changes bigger than the noise.


---

- finished: 2026-10-28T20:44:26+00:00
- duration: 31.78s
- status: ok
- input_tokens: 8
- output_tokens: 2656
- cache_read_tokens: 89270
- cache_write_tokens: 15209
- cost_usd: 0.192678
