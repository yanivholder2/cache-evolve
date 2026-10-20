# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/variants/v001
- allow_edits: True
- started: 2026-10-19T14:11:10+00:00

## timeline

### 14:11:18 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles; cat 0007/hypotheses/H01.md; echo ----; cat 0007/variants/v001/eval/fitness-vector.json; echo ----; cat 0006/variants/v002/eval/fitness-vector.json; echo ----; cat 0006/variants/v002/insight.md; echo ----; ls -R 0007/variants/v001 | head -50; echo; diff -r 0006/variants/v002/attempts/01/overlay 0007/variants/v001/attempts/01/overlay",
  "description": "Read hypothesis, fitness vectors, insight, and diff overlays"
}
```

### 14:11:20 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/hypotheses/H01.md"
}
```

### 14:11:20 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/variants/v001/eval/fitness-vector.json"
}
```

### 14:11:21 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0006/variants/v002/eval/fitness-vector.json"
}
```

### 14:11:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0006/variants/v002/insight.md"
}
```

### 14:11:24 thinking



### 14:11:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 14:11:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/variants/v001/attempts/01/overlay/src/dash/segments/mod.rs"
}
```

### 14:11:34 thinking



### 14:11:45 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/variants/v001/insight.md",
  "content": "# Variant v001 Insight\n\n## Helped\n\n- hit-rate: Hit rate rose to exactly the value the hypothesis predicted for cycle-6 v001's rescue. A stash victim from `stash_insert_with_rescue_hashed` is now placed in a free slot of its target or probing bucket instead of being dropped. All three stash insert sites go through the wrapper: the fresh `put` and both inserts in `demote_from_normal_bucket`. Matching the prediction exactly is the bookkeeping check the hypothesis asked for. The ported rescue behaves like the original, and keeping the `borrowed` counter exact does not change placement. Rescue writes only into normal buckets, so it never changes `borrowed`.\n- per-op-time (get p50, relative to cycle-6 v001 only): get p50 landed below the rescue-only baseline named in the hypothesis. The `borrowed[s1] != 0` gate in `promotion_get` still skips some `s1` scans when the cache also uses rescue. The margin is small, though, so the hypothesis's \"clearly below\" criterion is met only weakly.\n\n## Hurt\n\n- per-op-time (get p50 vs parent): get got slower than v002 on the median. This change does not touch `promotion_get`, so the cost comes from the cache state that rescue creates. Normal buckets hold more items, so target and probing scans cost more on misses. The likely mechanism, which the evaluation cannot isolate, is that this partly cancels the `s1` skip the parent gained.\n- per-op-time (get p99): get's tail grew sharply. With rescue filling normal buckets, a stash hit's `promote_to_target` is more likely to evict. That eviction runs the whole `demote_from_normal_bucket` → stash insert → rescue chain inside the `get` call. The cascade branch also hashes the cascaded item again through `stash_insert_with_rescue`.\n- per-op-time (put p50 and put p99): put got slower. Its tail rose above v002 instead of below it, which **refutes** the hypothesis's put-p99 criterion. Removing the redundant hashes did help: `put_stash_at_index_known` takes `s0` as an argument, and the evictee's single `key_hash` serves both its counter decrement and its rescue target. But rescue adds more work to the same slow path. On every stash overflow, `put` now checks whether two normal buckets are full and does a `put_at_normal_idx` insert. That costs more than the rehash it saved.\n\n(metadata-size did not change: `Item` is still 40 bytes and `Dash` is the same size. Rescue and the hash-passing refactor add no fields.)\n\n## Takeaway\n\nOne hash per key really does let the counter and rescue share work: hit rate matched exactly, and the evictee's hash serves both purposes. But on its own terms, it does not make rescue cheap. The cost of rescue is not hashing. It is the extra normal-bucket work on the stash-overflow path, plus the fuller normal buckets it leaves behind, which raise both get tails and put tails. To keep rescue's hit-rate gain, the next change has to cut that extra work, for example by rescuing only into target, or by limiting how often rescue fills normal buckets. Trimming more hashes will not be enough.\n"
}
```

### 14:11:52 chat

I wrote the insight for v001. It got the hit-rate gain it aimed for, but it did not get the speed it hoped for, so the hypothesis is only partly confirmed.

- **Hit rate:** it rose to exactly the value the hypothesis predicted. Keeping the borrowed counter exact and adding stash-victim rescue don't change each other's placement decisions, which was the bookkeeping check the hypothesis asked for.
- **get p50:** it is below the rescue-only baseline the hypothesis compared against, so that criterion is met, but only barely. It is slower than the parent v002. This change doesn't touch the lookup code. My best explanation is that rescue leaves the normal buckets fuller, so scans on a miss cost more; the evaluation data can't isolate that cause.
- **get p99:** it rose sharply. A stash hit now more often evicts from a full target bucket, which runs the whole demote, stash insert and rescue chain inside the `get` call.
- **put p99:** it ended up above the parent, which refutes the hypothesis on this point. Removing the duplicate hashing did help. But rescue adds two bucket-full checks and an extra insert on the same slow path where a full stash evicts an item, and that costs more than the hashing it saved.
- **Metadata size:** unchanged.

**Takeaway:** sharing one hash per key between the counter and rescue works, but hashing was never what made rescue expensive. The cost is the extra bucket work rescue does and the fuller buckets it leaves behind. Rescue needs to do less work, for example by placing victims only in their target bucket, rather than having more hashes trimmed.

The file is `cycles/0007/variants/v001/insight.md`.


---

- finished: 2026-10-19T14:11:52+00:00
- duration: 42.66s
- status: ok
- input_tokens: 10
- output_tokens: 3227
- cache_read_tokens: 137594
- cache_write_tokens: 47728
- cost_usd: 0.473923
