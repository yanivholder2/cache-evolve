# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008
- allow_edits: True
- started: 2026-10-28T20:50:20+00:00

## timeline

### 20:50:24 tool: Bash

```json
{
  "command": "for f in variants/v00{1,2,3}/insight.md hypotheses/H0{1,2,3}.md; do echo \"=== $f\"; cat $f; done",
  "description": "Read variant insights and hypotheses"
}
```

### 20:50:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/variants/v001/insight.md"
}
```

### 20:50:27 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/variants/v002/insight.md"
}
```

### 20:50:28 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/variants/v003/insight.md"
}
```

### 20:50:28 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/hypotheses/H01.md"
}
```

### 20:50:29 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/hypotheses/H02.md"
}
```

### 20:50:29 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/hypotheses/H03.md"
}
```

### 20:50:32 thinking



### 20:50:43 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0008/cycle-insight.md",
  "content": "# Cycle 0008 Insight\n\n## Cohort outcome\n\nTwo variants branched from cycle-0007 v002 (capacity/2 ghost at 131K) and made the ghost smaller. v001 narrowed the large-size ghost to capacity/4. v002 cut the ghost tags from 16 bits to 8. v003 branched from the ghost-free cycle-0004 v002 and widened probation to about 1/3, at 131K only. All three are Winners. v001 is the hit-rate gain: a small summary rise, plus timing gains that are probably partly noise. v002 is the clearest latency result so far: get p50 improved with bit-identical hit-rate. v003 is a weak Winner: neutral to slightly negative on the summary, and most useful as a noise-floor and negative control.\n\n## Patterns\n\n- The ghost table's byte footprint slows get, even though get never reads the table. Both variants that shrank the ghost (v001 fewer slots, v002 narrower tags) had lower per-op times on unchanged get code. v002 changed only the tag width, so it is the first variant to isolate a cause of the long-running get-p50 regression. The footprint suspect from cycles 6–7 is now the leading explanation, not a guess.\n- At 131K, hm_0 and web_0 pull in opposite directions on any probation-time lever. A narrower ghost (v001) and a static wider probation (v003) both traded hits between the two traces. Only the ghost-targeted version came out net positive. This repeats cycle 5's result at 16K: selective reprieves beat a uniformly wider window at both sizes.\n- Ghost width tuning is nearly used up. capacity/4 is slightly better than capacity/2, and the moves on both traces are now marginal.\n- The noise floor is now visible. v003 left the hot paths identical, yet its per-op times moved noticeably. Any per-op delta of similar size in v001 or v002 should not count as evidence on its own. v002's get-p99 rise has no code cause and fits this noise.\n- v002's hit-rate was bit-identical even though 8-bit tags collide far more often. This is plausible, because spurious reprieves are cheap. It could also mean the harness never exercises those collisions. This has not been checked.\n\n## Open questions\n\n- Do v001 and v002 stack? A capacity/4 ghost with 8-bit tags would be a quarter of the parent's ghost bytes at 131K. Does get p50 keep falling?\n- Can the ghost get smaller still, for example with 4-bit tags, a bitset or a Bloom-style ghost, before the hit-rate cost becomes visible?\n- Are spurious reprieves actually happening in the hit-rate harness? Count them.\n- Is v002's get-p50 gain larger than v003's noise swing when measured in repeated paired runs?\n\n## Continuation\n\nMake the base cycle-0008 v001 combined with v002's 8-bit tags (capacity/4 ghost, u8 tags). Keep shrinking ghost bytes as the latency lever, and add a spurious-reprieve counter to confirm hit-rate neutrality. Drop static probation/protected split retuning at every size, and drop further ghost-width tuning. Any new hit-rate lever at 131K should change how reprieves are granted, not the window size. Report get-p50 claims against v003's noise floor and confirm them with paired runs.\n"
}
```

### 20:50:50 chat

I wrote `cycle-insight.md` in the cycle 0008 output dir. All three variants are listed as Winners, but only v001 and v002 actually gained; v003 came out neutral to slightly negative on hit-rate.

- **v002 (8-bit ghost tags):** the clearest latency result so far. Get p50 dropped although get code was untouched, and hit-rate was bit-identical. This is the first direct evidence that the ghost table's memory footprint slows get, even though get never reads it.
- **v001 (capacity/4 ghost at 131K):** a small summary hit-rate gain. hm_0 recovered and web_0 gave back slightly less. All four timings also improved, but some of that is probably noise.
- **v003 (wider probation at 131K, no ghost):** traded hits between the two traces and lost slightly overall. That is the cycle-5 result again: giving extra time only to returning keys beats widening the window for every key. Its get/put code was unchanged, yet its timings moved noticeably, so it sets a rough noise floor for this cycle's per-op deltas.

The Continuation section tells the next cycle to:
- start from v001 combined with v002's 8-bit tags;
- keep shrinking ghost bytes as the latency lever;
- add a counter for false ghost matches (reprieves granted by a tag collision);
- drop probation-split and ghost-width tuning;
- check get-p50 claims against v003's noise floor and confirm them with repeated paired runs.

One thing is unconfirmed: v002's hit-rate stayed bit-identical even though 8-bit tags should collide far more often. The cost may really be negligible, or the hit-rate harness may never hit those collisions; nobody has checked, so it's listed as an open question.


---

- finished: 2026-10-28T20:50:50+00:00
- duration: 30.65s
- status: ok
- input_tokens: 8
- output_tokens: 2643
- cache_read_tokens: 89076
- cache_write_tokens: 14916
- cost_usd: 0.190035
