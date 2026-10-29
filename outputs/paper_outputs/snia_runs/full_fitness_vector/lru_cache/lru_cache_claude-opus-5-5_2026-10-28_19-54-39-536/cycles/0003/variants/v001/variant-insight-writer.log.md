# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v001
- allow_edits: True
- started: 2026-10-28T20:12:49+00:00

## timeline

### 20:12:53 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles; cat 0003/hypotheses/H01.md; echo ---; find 0003/variants/v001 0002/variants/v003 -type f | head -50; echo ---; cat 0003/variants/v001/eval/fitness-vector.json; echo ---; cat 0002/variants/v003/eval/fitness-vector.json; echo ---; cat 0002/variants/v003/insight.md",
  "description": "Read hypothesis, file list, fitness vectors, parent insight"
}
```

### 20:12:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/hypotheses/H01.md"
}
```

### 20:12:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v001/eval/fitness-vector.json"
}
```

### 20:12:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/variants/v003/eval/fitness-vector.json"
}
```

### 20:12:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/variants/v003/insight.md"
}
```

### 20:13:00 thinking



### 20:13:01 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v001"
}
```

### 20:13:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v001/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 20:13:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v001/manifest.toml"
}
```

### 20:13:13 thinking



### 20:13:23 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0003/variants/v001/insight.md",
  "content": "# Variant v001 Insight\n\n## Helped\n\n- hit-rate (hm_0, both sizes): Here the ghost does what the hypothesis intended. A hm_0 key that falls off the 20% probation segment before its second reference is recognised when it comes back and goes straight to protected. The gain is largest at the small size, where probation is too short to catch medium reuse distances.\n- metadata-size: `Node<u64,u64>` stays at 24 bytes because the ghost lives in per-cache `HashSet`/`VecDeque` fields, not in the node. The summary dimension is unchanged, as step 1 of the hypothesis required.\n\n## Hurt\n\n- hit-rate (web_0, both sizes): The prediction was backwards. web_0 at the large size, which was meant to gain the most, lost the most. Every ghost hit is inserted at protected MRU and calls `demote_if_over`, which pushes the protected LRU out to probation. With `ghost_cap = capacity/2`, web_0's long-distance returners keep displacing protected entries that were proven hot by a real resident hit. The result is a net loss, and the gains on hm_0 only just offset it in the summary.\n- per-op-time (put): Every miss now pays for a `ghost_set.remove(&key)` lookup. Every probation eviction also pays for a key clone, a `HashSet` insert, a `VecDeque` push, and an overflow pop with a second `HashSet` remove. That adds about three hash operations to a miss path that already does a map remove and insert.\n- per-op-time (get): `get` p50 and p99 rose even though the code path is unchanged. The likely cause is memory, not instructions. A ghost `HashSet` of up to half the capacity, plus its FIFO, competes with `map` and `nodes` for CPU cache, so the same hash lookup and relink in `get` miss cache more often. The summary metric is get p50, so this is the costliest effect of the change.\n- metadata-size (secondary, `LruCache` struct): The header grew to hold the `HashSet`, the `VecDeque` and `ghost_cap`. This is a fixed cost per cache and does not count toward the summary, but it adds to the secondary row.\n\n## Takeaway\n\nAn A1out ghost that sends a returning key directly to protected MRU is too aggressive for web_0. On that trace a second reference at long distance is weaker evidence than a resident hit, yet this variant ranks the two equally and demotes protected entries to make room. The ghost also costs far more than \"no hit-path cost\": a separate hashed structure of half the capacity slows `get` through cache pressure even when `get` never touches it. The summary hit-rate barely moved. A ghost is worth keeping only if it is much smaller or cheaper, for example fingerprints instead of full keys or a smaller `ghost_cap`. It also needs a gentler admission rule, such as probation MRU or protected without forced demotion.\n"
}
```

### 20:13:33 thinking



### 20:13:33 chat

I've written `insight.md` for v001 in the output dir. Hypothesis H01 did not hold up. The bounded ghost history barely changed the overall hit-rate, and it slowed the cache down.

- **Wrong direction on web_0:** the hypothesis said the big gain would be on web_0 at the large size. That spot lost the most instead. Each returning ghost key goes straight into the protected segment and pushes out an entry that had a real hit while cached. With the ghost set to half the capacity, this happens often enough on web_0 to lose more than it gains.
- **hm_0 improved at both sizes,** most at the small size. The ghost there catches keys that fall out of the short probation segment just before they're used again. The hm_0 gains only just make up for the web_0 losses.
- **Slower puts:** each miss now does about three extra hash operations for the ghost.
- **Slower gets, though the get code didn't change:** get speed is the summary timing metric, so this is the most costly effect. My guess is that the extra ghost structures crowd the CPU cache. The data doesn't confirm that.
- **Node size stayed at 24 bytes,** as the hypothesis required. Only the cache's fixed header grew.

The takeaway suggests a smaller or cheaper ghost, such as storing fingerprints instead of full keys. It also suggests a gentler rule for returning keys, like putting them back in probation or not forcing a demotion.

I couldn't use shell or file search in this session, so I didn't list the `eval/` folder. There was also no `manifest.toml` in the variant dir. The insight is based on the hypothesis, the two fitness vectors, the parent's insight and v001's `src/lru_cache/mod.rs`.


---

- finished: 2026-10-28T20:13:33+00:00
- duration: 44.28s
- status: ok
- input_tokens: 12
- output_tokens: 3579
- cache_read_tokens: 140672
- cache_write_tokens: 23547
- cost_usd: 0.288138
