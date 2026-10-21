# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0003/variants/v001
- allow_edits: True
- started: 2026-10-21T09:22:44+00:00

## timeline

### 09:22:50 tool: Bash

```json
{
  "command": "C=/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles; cat $C/0003/hypotheses/H01.md; echo ---; cat $C/0003/variants/v001/eval/fitness-vector.json; echo ---; cat $C/0001/variants/v001/eval/fitness-vector.json; echo ---; cat $C/0001/variants/v001/insight.md; echo ---; ls -R $C/0003/variants/v001 $C/0001/variants/v001/attempts/01/overlay; cat $C/0003/variants/v001/manifest.toml 2>/dev/null",
  "description": "Read hypothesis, fitness vectors, parent insight, listings"
}
```

### 09:22:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0003/hypotheses/H01.md"
}
```

### 09:22:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0003/variants/v001/eval/fitness-vector.json"
}
```

### 09:22:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0001/variants/v001/eval/fitness-vector.json"
}
```

### 09:22:54 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0001/variants/v001/insight.md"
}
```

### 09:22:54 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0003/variants/v001/manifest.toml"
}
```

### 09:22:56 chat

Now the source files.

### 09:22:57 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0003/variants/v001"
}
```

### 09:22:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0003/variants/v001/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 09:23:00 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0001/variants/v001/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 09:23:05 thinking



### 09:23:10 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0003/variants/v001/insight.md",
  "content": "# Variant v001 Insight\n\n## Helped\n\n- hit-rate: Cutting `ghost_cap` from `capacity - small_target` to `capacity / 2` (at size 2048, from 1843 remembered keys to 1024) means a key evicted from `small` must come back within a much shorter window of later small-queue evictions to count as a ghost hit. Keys that come back from loops longer than that window now re-enter `small` and must prove reuse in probation. Before, they went straight into `main`, where they displaced resident entries that were being reused.\n\n## Hurt\n\n(No dimension regressed. The fitness vector reports only `hit-rate`. The smaller ghost should also roughly halve the `ghost_q`/`ghost_set` footprint and its `with_capacity` reservations, but `metadata-size` and `per-op-time` were not measured, so that gain is unverified.)\n\n## Takeaway\n\nThe ghost window size is a real tuning knob on workload-a, and the parent's near-capacity ghost was too loose. Admitting a ghost hit straight into `main` only helps when the window is short enough that a hit actually signals near-term reuse. Since shrinking the window helped and nothing else in the policy changed, the next probe on this axis should go smaller still (e.g. `capacity / 4`) to find where too few genuine re-references are caught. Fingerprinting or other structural ghost changes should wait until the size optimum is found.\n"
}
```

### 09:23:16 chat

I wrote `insight.md` for v001. The hypothesis held: making the ghost window smaller raised hit-rate on workload-a.

- **Change:** The only policy change is in `try_new`: `ghost_cap` went from `capacity - small_target` to `capacity / 2`. At size 2048 that means the cache remembers 1,024 recently evicted keys instead of 1,843. The variant also adds a new test (`ghost_window_expires_after_ghost_cap_plus_one_evictions`) and a few extra assertions in existing ones.
- **Helped (hit-rate):** A key evicted from the small queue now has to come back sooner to skip straight into `main`. Keys that return from long loops go back through probation instead, so they no longer push out entries that are actually being reused.
- **Hurt:** Nothing got worse. The smaller ghost should roughly halve its memory use, but that wasn't measured: the fitness vector only reports hit-rate, so the insight says this is unverified.
- **Takeaway:** Ghost size is a real tuning knob, and the parent's near-capacity window was too loose. I suggested trying an even smaller window next (e.g. `capacity / 4`) to find the best size before changing how the ghost is stored.

`manifest.toml` doesn't exist in the variant directory, so I didn't use it. I also couldn't list the `eval/` directory, so I read only `fitness-vector.json` from it.


---

- finished: 2026-10-21T09:23:16+00:00
- duration: 31.82s
- status: ok
- input_tokens: 12
- output_tokens: 2599
- cache_read_tokens: 131251
- cache_write_tokens: 19945
- cost_usd: 0.237838
