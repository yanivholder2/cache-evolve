# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0001/variants/v002
- allow_edits: True
- started: 2026-10-28T14:18:27+00:00

## timeline

### 14:18:32 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve; cat outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0001/hypotheses/H02.md; V=outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0001/variants/v002; ls -R $V | head -50; cat $V/manifest.toml $V/eval/fitness-vector.json; B=inputs/subjects/associative_cache/baseline; cat $B/eval/profiles/msr/fitness-vector.json $B/insight.md; ls -R $B/overlay"
}
```

### 14:18:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0001/hypotheses/H02.md"
}
```

### 14:18:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0001/variants/v002/eval/fitness-vector.json"
}
```

### 14:18:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/associative_cache/baseline/eval/profiles/msr/fitness-vector.json"
}
```

### 14:18:36 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/associative_cache/baseline/insight.md"
}
```

### 14:18:36 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0001/variants/v002/manifest.toml"
}
```

### 14:18:39 thinking



### 14:18:39 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0001/variants/v002"
}
```

### 14:18:40 tool: Glob

```json
{
  "pattern": "**/eviction_policy.rs",
  "path": "/home/user/cache-evolve/inputs/subjects/associative_cache/baseline"
}
```

### 14:18:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0001/variants/v002/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 14:18:43 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/associative_cache/baseline/overlay/src/shared/eviction_policy.rs"
}
```

### 14:18:51 thinking



### 14:18:56 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0001/variants/v002/insight.md",
  "content": "# Variant v002 Insight\n\n## Helped\n\n(none — on web_0 at the larger size hit-rate stayed about level, which is noise, not a gain)\n\n## Hurt\n\n- hit-rate: Inserts still land at the MRU tail through the unchanged `Bucket::put`, but a hit now only moves an item up `len/8` slots. So every brand-new key outranks a key that was just re-referenced, which inverts the frequency gradient the hypothesis wanted. A reused item that has aged near index 0 gets only a short lift and is still evicted soon. The loss is largest on hm_0 at the larger cache size, where the baseline's full jump to MRU was holding on to a working set with moderate reuse distance.\n\n## Takeaway\n\nLimiting how far a hit can promote an item, while new items are still inserted at the very top, punishes reuse instead of rewarding it. On both MSR traces, bounded promotion did worse than the classic jump to MRU. The hit-rate drop was flat-to-small on web_0 and clear on hm_0. A graded-promotion scheme only makes sense if new items are also inserted lower in the bucket (for example at or below the promotion step), so that a hit is worth more than an insert. Per-op-time was not measured for this variant, so the side benefit of shorter memmoves claimed in the hypothesis is still untested.\n"
}
```

### 14:19:01 thinking



### 14:19:01 chat

I wrote `insight.md` for v002. The variant's change lowered hit-rate rather than raising it.

**What changed:** in `ClassicLRU::get_from_position`, a hit used to jump the item straight to the MRU end. Now it moves up only `len/8` slots. Buckets of 8 items or fewer keep the old full jump.

**Result:** hit-rate fell. The drop was small or flat on web_0 and clear on hm_0, worst at the larger cache size.

**Why it fell:** new keys still enter at the MRU end through the unchanged `Bucket::put`. A hit now earns a smaller lift than an insert, so a just-reused key sits below every brand-new key. That is the reverse of the frequency ordering the hypothesis wanted.

**Takeaway in the insight:** limiting how far a hit can promote an item only helps if new items are also inserted lower in the bucket. Per-op-time wasn't measured for v002, so the faster-gets benefit the hypothesis expected is still untested.

I couldn't find the parent's `eviction_policy.rs` in the baseline overlay. I took the old full-jump behaviour from the hypothesis's description instead of comparing the two files directly.


---

- finished: 2026-10-28T14:19:02+00:00
- duration: 34.27s
- status: ok
- input_tokens: 12
- output_tokens: 2746
- cache_read_tokens: 131829
- cache_write_tokens: 16233
- cost_usd: 0.211198
