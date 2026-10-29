# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/variants/v003
- allow_edits: True
- started: 2026-10-28T15:25:59+00:00

## timeline

### 15:26:03 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632; cat cycles/0008/hypotheses/H03.md; echo ----; cat cycles/0008/variants/v003/eval/fitness-vector.json; echo ----; cat cycles/0007/variants/v001/eval/fitness-vector.json; echo ----; cat cycles/0007/variants/v001/insight.md; echo ----; ls -R cycles/0008/variants/v003 cycles/0007/variants/v001 | head -80",
  "description": "Read hypothesis, fitness vectors, parent insight, list files"
}
```

### 15:26:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/hypotheses/H03.md"
}
```

### 15:26:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/variants/v003/eval/fitness-vector.json"
}
```

### 15:26:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/variants/v001/eval/fitness-vector.json"
}
```

### 15:26:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/variants/v001/insight.md"
}
```

### 15:26:07 thinking



### 15:26:09 tool: Grep

```json
{
  "pattern": "CRP_DIV|probation_age|is_correlated_probation_hit",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/variants/v003/attempts/01/overlay",
  "output_mode": "content",
  "-n": "true",
  "-C": "3"
}
```

### 15:26:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/variants/v003/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 15:26:19 thinking



### 15:26:26 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/variants/v003/insight.md",
  "content": "# Variant v003 Insight\n\n## Helped\n\n- hit-rate (web_0 @ 131072): This is the largest gain the lineage has had in a single cell since probation was added, and it passes the hypothesis's bar easily. In 1024-slot buckets the window covers the newest 16 probation insertions. That is wide enough to catch web_0's back-to-back and burst re-references, which until now earned a main slot from a single early hit. Those blocks now reach the FIFO head with counter 0 and are evicted, so they no longer displace real main residents. This supports cycle 0007's finding that never-re-hit, counter-0 promotions made up most of main's victims.\n- hit-rate (hm_0 @ 16384, hm_0 @ 131072): Both cells rose slightly instead of falling. This answers the refutation clause's concern: on hm_0, short-gap reuse is not real reuse. Its long-distance re-references arrive after the window closes and still promote, and the few burst promotions removed from main free a little space.\n\n## Hurt\n\n- hit-rate (web_0 @ 16384): Flat to slightly down, so it misses the hypothesis's bar at this size. In 128-slot buckets the window is only `64 / 16 = 4` insertions. Most of a burst's later hits land after the window and still bump the counter and promote. So the filter catches almost none of the correlated promotions here, while it occasionally withholds the bump from a true immediate re-use.\n\n## Takeaway\n\nFiltering correlated references is the first change to probation's *promotion* signal that has worked, and its effect scales with the window's absolute width, not with `small_len / 16` as a ratio. At 16 slots it removes burst-driven promotions on web_0 without costing hm_0 anything. At 4 slots it is too narrow to matter. The obvious next step is to give 128-slot buckets a wider window, for example an absolute floor of about 16 slots or a smaller divisor, rather than drop the idea. The hm_0 result shows there is headroom before real reuse starts being filtered.\n"
}
```

### 15:26:31 thinking



### 15:26:31 chat

I wrote `insight.md` for v003 in `cycles/0008/variants/v003/`. H03 half-worked. The change stops a probation hit that comes right after insertion from counting toward promotion into main.

- **web_0 at size 131072:** this is the cell that gained. In 1024-slot buckets the window covers the newest 16 insertions, which is wide enough to catch web_0's bursts. Blocks hit only during a burst no longer get into main, so they stop pushing out real residents.
- **hm_0 at both sizes:** both went up slightly. That answers the refutation clause: on hm_0, quick repeat hits aren't real reuse.
- **web_0 at size 16384:** flat to slightly down, so it misses the hypothesis's bar. In 128-slot buckets the window is only 4 insertions, so most of a burst's later hits still land after it and still promote.

**Takeaway:** what matters is how wide the window is in slots, not the `small_len / 16` ratio. The next variant could set a minimum of about 16 slots, or use a smaller divisor, for 128-slot buckets. hm_0 suggests a wider window won't start filtering its real reuse yet.


---

- finished: 2026-10-28T15:26:32+00:00
- duration: 32.43s
- status: ok
- input_tokens: 12
- output_tokens: 2751
- cache_read_tokens: 140058
- cache_write_tokens: 22246
- cost_usd: 0.261048
