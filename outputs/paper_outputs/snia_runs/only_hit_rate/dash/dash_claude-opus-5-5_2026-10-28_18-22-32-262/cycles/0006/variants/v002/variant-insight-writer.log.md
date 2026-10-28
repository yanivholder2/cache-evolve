# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006/variants/v002
- allow_edits: True
- started: 2026-10-28T19:08:49+00:00

## timeline

### 19:08:53 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262; cat cycles/0006/hypotheses/H02.md; echo ----; cat cycles/0006/variants/v002/eval/fitness-vector.json; echo ----; cat cycles/0005/variants/v003/eval/fitness-vector.json; echo ----; cat cycles/0005/variants/v003/insight.md; echo ----; ls -R cycles/0006/variants/v002 cycles/0005/variants/v003 | head -50",
  "description": "Read hypothesis, fitness vectors, parent insight"
}
```

### 19:08:55 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006/hypotheses/H02.md"
}
```

### 19:08:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006/variants/v002/eval/fitness-vector.json"
}
```

### 19:08:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0005/variants/v003/eval/fitness-vector.json"
}
```

### 19:08:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0005/variants/v003/insight.md"
}
```

### 19:08:59 thinking



### 19:09:00 tool: Grep

```json
{
  "pattern": "GHOST_FACTOR|filter|cycle 0006|fn remove|fn insert",
  "path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006/variants/v002/attempts/01/overlay",
  "output_mode": "content",
  "-n": "true"
}
```

### 19:09:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006/variants/v002/attempts/01/overlay/src/dash/segments/dash_segment.rs",
  "limit": 200
}
```

### 19:09:05 thinking



### 19:09:12 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0006/variants/v002/insight.md",
  "content": "# Variant v002 Insight\n\n## Helped\n\n- (none) No trace/size cell went up. The cell the hypothesis targeted, `web_0` at 16K, is the one that fell the most.\n\n## Hurt\n\n- hit-rate (16K, `web_0`): This is the largest drop, and it hit the very cell the change was meant to lift. A fingerprint now lasts four times as many evictions, so a lot of keys from long ago come back still holding ghost credit. Each one is promoted on its first stash hit and pushes out a resident that was more useful. So `web_0`'s returning keys at 16K were never just under the old ring's reach. Most of them return too late to be worth caching, and stretching the ring only credits more of them.\n- hit-rate (16K, `hm_0`): A small loss. This is the stale-credit risk the rationale named. Every extra early promotion costs little, as v003 showed, but they add up to a slight net loss rather than holding level.\n- hit-rate (128K, `web_0` and `hm_0`): Both cells slipped by a small amount. At 128K the stash is larger, so the factor-2 ring already covered the short reuse window that v003 had gained from. The extra reach adds only older, weaker evidence.\n- The counting pre-filter does not change behaviour. It only lets `remove` skip the scan for absent fingerprints, and it never gives a false negative. So all the movement in these cells comes from the reach change alone.\n\n## Takeaway\n\nMaking the ghost ring longer is not a lever that helps. On `web_0` at 16K, the extra evictions it remembers cost more than they return, because promotions on old credit push out better residents. So v003's short ring was not the 16K bottleneck. The factor of 2 was already close to the useful reach, and the flat `web_0` 16K cell has to be fixed somewhere other than ghost span: placement or demotion-side credit, not memory length.\n"
}
```

### 19:09:19 thinking



### 19:09:19 chat

I wrote `insight.md` for v002. The hypothesis didn't hold: raising `GHOST_FACTOR` from 2 to 8 lowered hit-rate slightly in all four trace/size cells, so mean hit-rate came in below the parent's 0.40195 rather than above it.

- **Target cell:** `web_0` at 16K, the cell the change was meant to lift, fell the most. My reading is that a longer ring lets keys evicted long ago keep their ghost credit. Each one is then promoted on its first stash hit and pushes out a more useful resident.
- **Other cells:** `hm_0` at 16K dropped a little, which is the stale-credit risk the hypothesis named. Both 128K cells slipped slightly, since the original ring already covered the short reuse window that v003 gained from.
- **Pre-filter:** both steps of the hypothesis were implemented, the factor change and the counting pre-filter. The filter only lets the ring skip a scan when a key is clearly absent and never changes which keys get credit, so all the movement comes from the longer reach.

**Takeaway:** the short ring wasn't what held back `web_0` at 16K, and making it longer doesn't help. That cell needs a fix elsewhere, such as placement or giving ghost credit on demotion (the ideas behind H01 and H03), not a longer memory.

I inferred the mechanism from the code change and the per-cell results. The eval doesn't measure stale promotions directly, so that explanation is a reading, not a measurement.


---

- finished: 2026-10-28T19:09:19+00:00
- duration: 29.91s
- status: ok
- input_tokens: 12
- output_tokens: 2395
- cache_read_tokens: 131077
- cache_write_tokens: 13826
- cost_usd: 0.184771
