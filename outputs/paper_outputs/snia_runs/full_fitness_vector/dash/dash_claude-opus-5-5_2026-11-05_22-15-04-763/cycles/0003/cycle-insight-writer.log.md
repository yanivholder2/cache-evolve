# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003
- allow_edits: True
- started: 2026-11-05T22:38:52+00:00

## timeline

### 22:38:57 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003; for f in variants/v00*/insight.md hypotheses/H0*.md; do echo \"=== $f\"; cat \"$f\"; done",
  "description": "Read variant insights and hypotheses"
}
```

### 22:38:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/variants/v001/insight.md"
}
```

### 22:39:00 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/variants/v002/insight.md"
}
```

### 22:39:00 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/variants/v003/insight.md"
}
```

### 22:39:01 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/hypotheses/H01.md"
}
```

### 22:39:02 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/hypotheses/H02.md"
}
```

### 22:39:02 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/hypotheses/H03.md"
}
```

### 22:39:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0003/cycle-insight.md",
  "content": "# Cycle 0003 Insight\n\n## Cohort outcome\n\nAll three variants were evaluated. Two built on cycle 0002's v003 (two-hit gate plus MRU demotion): v001 reordered the lookup to check the normal tier before stash, and v002 added probing overflow. The third, v003, applied MRU demotion to the baseline with no gate. There are no Winners. v002 was the best result, with a very small hit-rate gain on every cell and better tails on both `put` and `get`. v003 showed that MRU demotion raises hit rate even without the gate, but it did not recover baseline latency.\n\n## Patterns\n\n- Lookup order is not the lever for `get` latency. Searching stash last (v001) left hit rate unchanged and made `get` p50 and p99 worse. Stash hits make up a large enough share of all hits that stash-first is the better order.\n- The `get` slowdown follows stash-hit traffic, not the gate. Ungated MRU demotion (v003) reached the same slower `get` band as the gated variants. The likely cost is the stash-hit promotion chain (remove, promote, demote cascade), which runs more often when stash keeps items longer.\n- Promotion placement and demotion placement are not independent hit-rate levers. Once demoted items land at stash MRU, overflowing promotions into probing (v002) adds very little retention. What overflow does deliver is fewer demotion cascades, which reversed most of the `put` p99 regression caused by stash pressure. This is the clearest evidence so far that cascade count drives the tail.\n- MRU demotion's hit-rate gain is again largest on hm_0 at the large size, with or without the gate. That cell is consistently where retention changes decide the result.\n\n## Open questions\n\n- Can the stash-hit promotion path itself be made cheaper, for example by skipping or deferring the demotion cascade when it is not needed, without losing retention?\n- With v002's fewer cascades, how far is the gated, MRU-demoting line from dominating the baseline? Is the remaining gap only `get` p50?\n- Would probing overflow added to ungated MRU demotion (v003 here) recover enough tail latency to make that cheaper branch competitive?\n\n## Continuation\n\nTake v002 (gate + MRU demotion + probing overflow) as the new base. It has the best combined hit rate and tail profile so far. Aim latency work at the cost of each stash-hit promotion and at reducing demotion cascades further. Drop lookup-reordering ideas and stop treating the gate as the source of the `get` slowdown. Optionally try overflow on the ungated MRU branch as a cheaper alternative line.\n"
}
```

### 22:39:23 thinking



### 22:39:23 chat

I wrote the cycle 0003 insight to `cycle-insight.md` in the output dir. All three variants ran this cycle, but there are still no Winners.

- **v002 did best.** It added probing overflow on top of the two-hit gate and MRU demotion. Hit rate went up on every trace and size, though only by a very small amount. Tail latency improved on both `put` and `get`, and most of the earlier `put` p99 regression went away. The likely reason is that fewer items get demoted into stash, so fewer `put`s land on a full bucket.
- **Lookup order isn't the fix.** v001 searched the normal tier before stash. Hit rate didn't change, but `get` got slower at both p50 and p99. So checking stash first is the better order.
- **The gate isn't what slows `get` down.** v003 used MRU demotion without the gate. Hit rate still rose, so MRU demotion works on its own. But `get` was just as slow as the gated variants. The slowdown seems to come from how often a stash hit has to promote an item and push another one down. It isn't the gate.
- **The two placement changes don't add up much.** Once demoted items go to the stash MRU end, also changing where promotions land adds very little hit rate. Its real benefit is latency.

The note tells the next cycle to use v002 as the new base and to cut the cost of each stash-hit promotion and its knock-on demotions. It also says to drop lookup-reordering ideas and stop blaming the gate for slow `get`. Adding probing overflow to the version without the gate is listed as an optional cheaper line to try.


---

- finished: 2026-11-05T22:39:23+00:00
- duration: 31.16s
- status: ok
- input_tokens: 8
- output_tokens: 2257
- cache_read_tokens: 85485
- cache_write_tokens: 13762
- cost_usd: 0.172707
