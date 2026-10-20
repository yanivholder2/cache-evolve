# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/variants/v003
- allow_edits: True
- started: 2026-10-20T10:10:16+00:00

## timeline

### 10:10:19 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles; cat 0010/hypotheses/H03.md; echo ---; cat 0010/variants/v003/eval/fitness-vector.json; echo ---; cat 0009/variants/v001/eval/fitness-vector.json; echo ---; cat 0009/variants/v001/insight.md; echo ---; ls -R 0010/variants/v003 0009/variants/v001/attempts/01/overlay; cat 0010/variants/v003/manifest.toml 2>/dev/null"
}
```

### 10:10:20 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/hypotheses/H03.md"
}
```

### 10:10:21 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/variants/v003/eval/fitness-vector.json"
}
```

### 10:10:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0009/variants/v001/eval/fitness-vector.json"
}
```

### 10:10:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0009/variants/v001/insight.md"
}
```

### 10:10:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/variants/v003/manifest.toml"
}
```

### 10:10:24 thinking



### 10:10:25 tool: Grep

```json
{
  "pattern": "vbit_set|to_main|push_back_main",
  "path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/variants/v003/attempts/01/overlay",
  "-n": "true",
  "output_mode": "content"
}
```

### 10:10:26 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/variants/v003"
}
```

### 10:10:27 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/variants/v003/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 10:10:33 thinking



### 10:10:41 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/variants/v003/insight.md",
  "content": "# Variant v003 Insight\n\n## Helped\n\n(none) No dimension improved in a meaningful way. The `put` p99 dropped slightly and the `get` p99 rose slightly. This fits run-to-run noise: the change adds only one `vbit_set` on the ghost-hit branch of `put`, writing to a bitset word that the slot-reuse `vbit_clear` has just touched.\n\n## Hurt\n\n- hit-rate: Setting VISITED on ghost returners (`self.vbit_set(idx)` after `push_back_main`) dropped hit-rate below the parent. That is under the hypothesis's own 0.7314 failure threshold, so the hypothesis is refuted. A returner already gets roughly one full SIEVE lap of protection just by entering at the main tail. The pre-set bit doubles that to about two laps. So every returner that turns out to be one-shot now holds a main slot for an extra pass. While the hand spends its clearing pass on these returners, it evicts something else instead: older main nodes whose bit is clear. These include keys promoted from small, which enter main with VISITED cleared even though they earned a real hit.\n\n## Takeaway\n\nOn workload-a, a ghost hit is enough evidence to skip probation, but not enough to also count as a main-queue hit. Giving returners pre-set credit creates an asymmetry. A ghost returner, which has shown one reuse across an eviction, now outranks a promoted small key, which was actually hit while resident and loses its bit on promotion. The hand then sacrifices those promoted keys to protect returners that often do not come back within two laps. Cycle 0007 found that weakening admission (placing returners at the hand) also lost hit-rate. Together with this result, the current admission strength looks like a local optimum: plain tail insertion with VISITED clear. If returners are to be treated differently from promoted keys, it should come from evidence gathered while the key is resident, not from the ghost match alone.\n"
}
```

### 10:10:47 thinking



### 10:10:47 chat

I wrote the insight for v003, and the result is that H03 failed. Giving ghost returners their visited bit up front made the hit-rate worse than the parent's, dropping below the 0.7314 failure threshold the hypothesis set for itself.

- **Hit-rate (hurt):** A key that comes back from the ghost already gets about one full pass of the eviction hand just by entering at the back of the main queue. The pre-set bit makes that two passes. Returners that are never used again now hold a slot longer, and the hand evicts other nodes in their place. Those include keys promoted from the small queue, which were really hit but enter main with the bit cleared.
- **Per-op time and metadata size:** Metadata size didn't change. Get and put latency moved by amounts that look like noise. The change only adds one bit-set on the ghost-hit branch of `put`.

**Takeaway:** On workload-a, a ghost match is good enough reason to skip the small queue, but not to count as a hit as well. Cycle 0007 found that weakening admission also lost hit-rate. Together with this result, the current admission strength looks like the local optimum: returners go to the back of main with the bit cleared. Any extra credit for returners should come from hits they get while in the cache, not from the ghost match alone.

I couldn't run shell commands, and file search wasn't available either. So I didn't read `manifest.toml` (it doesn't exist) or the rest of `eval/`. The analysis rests on the hypothesis, both fitness vectors, the parent's insight and the v003 `mod.rs`. I couldn't produce a file diff, so I compared v003's code against the changes the hypothesis describes.

Files are in `cycles/0010/variants/v003`:
- insight.md


---

- finished: 2026-10-20T10:10:48+00:00
- duration: 31.34s
- status: ok
- input_tokens: 12
- output_tokens: 3044
- cache_read_tokens: 135723
- cache_write_tokens: 23378
- cost_usd: 0.275097
