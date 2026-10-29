# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009/hypotheses
- allow_edits: True
- started: 2026-10-29T14:21:16+00:00

## timeline

### 14:21:21 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280; cat cross-cycle-insights.md leaderboard.md cycles/0008/cycle-insight.md; cat cycles/0008/variants/*/insight.md; tail -c 3000 cycles-ledger.jsonl; ls cycles/0008/variants/*/attempts/*/overlay -R | head -50"
}
```

### 14:21:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cross-cycle-insights.md"
}
```

### 14:21:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/cycle-insight.md"
}
```

### 14:21:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/leaderboard.md"
}
```

### 14:21:27 thinking



### 14:21:28 tool: Glob

```json
{
  "pattern": "cycles/000[78]/variants/*/attempts/*/overlay/**/*.rs",
  "path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280"
}
```

### 14:21:29 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/cycle-winners.json"
}
```

### 14:21:31 thinking



### 14:21:32 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/variants/v001/insight.md"
}
```

### 14:21:32 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/variants/v001/attempts/01/overlay"
}
```

### 14:21:34 thinking



### 14:21:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0008/variants/v001/attempts/01/overlay/src/dash/segment.rs"
}
```

### 14:21:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/mod.rs"
}
```

### 14:21:38 thinking



### 14:21:38 chat

I'm reading the cycle history so the three hypotheses rest on what has been measured. I can't list directories in this session, so I'm using the insight files rather than the overlay source.

### 14:21:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles-ledger.jsonl"
}
```

### 14:21:51 thinking



### 14:21:51 chat

I couldn't list the overlay directories, so the plan below rests on the cycle insights. Writing the three hypotheses now.

### 14:22:00 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009/hypotheses/H01.md",
  "content": "# H01\n\nAn identical copy of incumbent cycle/0007/v003 will measure within a few ns of its recorded per-op-time (get p50 76), giving a noise control for every per-op-time verdict in this cohort.\n\n## Parent\n\ncycle/0007/v003\n\n## Changes\n\nThis is a deliberate noise-control variant. It has no behavioural change. Copy the parent's overlay unchanged. The only allowed edit is one comment line at the top of the Dash segment source file that holds the get-path `is_hottest` / `PROBE_WINDOW` logic, for example `// cycle 0009 H01: identical-copy noise control of cycle/0007/v003`. That gives the overlay a non-empty diff.\n\nDo NOT:\n- change any constant (`PROBE_WINDOW`, gate thresholds);\n- change any field, layout, or `#[inline]` attribute;\n- reorder any function;\n- add or remove any import.\n\nThe Item must stay 24 bytes and the per-segment `u32` tick must stay as it is. The window-throttled two-bucket swap on the full-probing arm must stay as it is. Hit-rate must reproduce the parent exactly in all four cells (web_0/hm_0 at 16384 and 131072). Any hit-rate difference means the copy is not identical and the variant is invalid.\n\n## Rationale (optional)\n\nFor seven cycles, every cycle insight has asked for an identical-baseline re-run, and none has been done. Uniform latency shifts that also hit untouched `put` paths have shown up in cycles 3 and 5 through 8. Because of them, no latency verdict can be trusted, and that includes cycle 0008 v001's apparent regression (p50 115).\n\nThis variant is timed in the same session and on the same host as H02 and H03. That gives the direct comparison the cycle 0008 Continuation asked for: incumbent copy vs. 0008 v001 descendants.\n\nHow to read the result:\n- **Confirmed:** get p50 lands within about ±5 ns of 76, and the p99 values land near the parent's. The benchmark is stable across sessions.\n- **Refuted:** a large uniform shift shows up on an unchanged binary. That proves the shift is environmental. Later cycles should then score latency only relative to a same-session control.\n"
}
```

### 14:22:09 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009/hypotheses/H02.md",
  "content": "# H02\n\nHalving `PROBE_WINDOW` on the eviction lineage will lower get p50 and get p99 (per-op-time) and leave hit-rate within ±0.0005 of the parent.\n\n## Parent\n\ncycle/0008/v001\n\n## Changes\n\nParent behaviour: on a hottest-position hit in a full probing bucket, the arm checks whether the hit is recent enough, using a tick-age test against `PROBE_WINDOW` and the per-segment `u32` tick. If it is, the arm promotes the item and evicts the probing LRU resident outright. The parent no longer uses the swap or the cascade.\n\nThe change:\n1. Set `PROBE_WINDOW` to half its current value, rounded down, with a minimum of 1. Change nothing else on that arm.\n2. Make sure the window bound is a `const`, or is computed once per segment or cache, and is never computed per get. Cycle 0008 v002 flagged a per-get bound computation as a possible cause of its median regression. If the parent computes the bound on every get, hoist it into a `const` or into a field set at construction time, without increasing the size of `Dash<u64,u64,DashSegment>` (currently 24 bytes). If neither option keeps that size, use a `const`.\n3. Keep the following unchanged:\n   - the full-target three-hit door guard;\n   - the two-hit stash gate;\n   - the two-hottest-hit full-probing guard;\n   - the single outright eviction;\n   - the 24-byte Item;\n   - the common target-hit path, with no new reads or writes.\n\n## Rationale (optional)\n\nCycle 0008 v002 showed that `PROBE_WINDOW` is a latency throttle, not a hit-rate lever. Doubling it gained nothing in hit-rate and raised get p50 and p99. The direction this hypothesis tests has not been tried yet: narrowing the window on the cheaper eviction arm, which cycle 0008 listed as an open question.\n\nOn that arm the promotion frees a slot instead of saturating buckets. A narrower window should therefore cost little or no hit-rate.\n\nHow to read the result: compare get p50 and p99 against H01, the same-session incumbent copy, and not against the cross-session numbers. The hypothesis is refuted if:\n- hit-rate drops by more than 0.0005, which would mean the arm's promotions carry real hits; or\n- get latency does not fall relative to H01.\n"
}
```

### 14:22:19 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009/hypotheses/H03.md",
  "content": "# H03\n\nStash victim selection with age-bounded protection will raise hit-rate in the large-cache cells (web_0-large and hm_0-large) without the small-cache losses that cycle 0008 v003's unbounded protection caused. The rule spares a reused stash entry only if its last hit is recent and only within the two LRU-end slots.\n\n## Parent\n\ncycle/0008/v001\n\n## Changes\n\nChange the stash victim routine: the code path that picks which stash entry leaves when an insert needs a slot in a full stash bucket. The parent evicts the entry at the LRU position. Cycle 0008 v003 changed this same routine. Replace that choice with a bounded, aged check:\n\n1. Look only at the two LRU-most stash slots. Call them `lru0`, the current victim, and `lru1`, the next one toward MRU. Do not scan the whole bucket. Cycle 0008 v003's put cost appeared on exactly this stash-full put path.\n2. Spare `lru0` and evict `lru1` instead only when all of these hold:\n   - `lru0.lfu_counter >= 1`, meaning it has been hit while in stash;\n   - `segment_tick.wrapping_sub(lru0.tick) < STASH_PROTECT_AGE`, meaning the hit was recent, using the per-segment `u32` tick already stored in Item;\n   - `lru1.lfu_counter == 0`.\n   In every other case, evict `lru0` exactly as the parent does.\n3. Add a new `const STASH_PROTECT_AGE: u32`. Set it to the stash bucket capacity times 4 ticks, or to the equivalent of about one stash lifetime given how the segment tick advances on inserts. Name the choice in a comment.\n4. When `lru0` is spared, it keeps its position. Do not move it to MRU and do not reset its counter. That way it gets one reprieve and ages out on the next pressure event after its window passes. This is the aging bound that cycle 0008 lacked.\n5. Constraints:\n   - Do not touch the common target-hit path.\n   - Do not add fields to Item. It must stay 24 bytes.\n   - Keep the two-hit stash gate, the full-target door guard, the probing guard, `PROBE_WINDOW`, and the single outright eviction exactly as they are in the parent.\n\n## Rationale (optional)\n\nCycle 0008 v003 protected never-hit-preferring stash entries with no aging. That filled the stash with stale one-hit keys and starved fresh arrivals, and hit-rate fell in every cell. Small caches were hurt most.\n\nThis variant addresses that failure in three ways:\n- A spare is allowed only for a *recent* hit.\n- Only one adjacent slot is considered.\n- A spared entry is never refreshed.\n\nAs a result, fresh arrivals are displaced at most one position earlier, and stale hits exit normally.\n\nHow to read the result:\n- **Confirmed:** large-cache web_0 or hm_0 rises by 0.001 or more, the small-cache cells stay within −0.0005, and put p99 stays near the same-session H01 control.\n- **Refuted:** a hit-rate loss in the small cells. The stash-exit lever is then dead, and cycles should stop proposing it.\n"
}
```

### 14:22:27 thinking



### 14:22:27 chat

I've written the three hypotheses for cycle 0009 to `cycles/0009/hypotheses/` as `H01.md` to `H03.md`. They're based only on the cycle insights and history files, not the variant source code. This session couldn't list directories, so I couldn't find or open the overlay files. The function and constant names in the `## Changes` sections are the ones the insights mention (`PROBE_WINDOW`, the `is_hottest` arm, `lfu_counter`, the per-segment tick). The implementer will need to match them to the real code.

- **H01 – parent cycle/0007/v003 (incumbent):** an identical copy whose only change is a comment, to serve as a noise control for timing. Seven cycle summaries have asked for this re-run and none was done. Unexplained uniform slowdowns, including on code paths a change never touched, have made recent latency verdicts unreliable. H02 and H03 will be timed alongside it, so their latency can be compared within the same session instead of across sessions.
- **H02 – parent cycle/0008/v001:** halve `PROBE_WINDOW` on the lineage that evicts the probing resident outright instead of swapping it, to cut get latency at no hit-rate cost. Cycle 8 widened the window and got no hit-rate gain and slower gets; narrowing it on this lineage hasn't been tried. It also moves the window-bound calculation out of the per-get path if the parent does it there.
- **H03 – parent cycle/0008/v001:** a capped version of the stash-exit change that failed in cycle 8. When the stash is full, the oldest entry is spared only if it was hit recently and the next-oldest entry has never been hit. The check covers just the two oldest slots and never refreshes the spared entry. The aim is a large-cache hit-rate gain without the small-cache losses cycle 8 saw. If it fails, that lever can be dropped for good.

H01 meets the requirement to use an incumbent as parent. H02 and H03 build on cycle/0008/v001, which the cycle 0008 summary recommended adopting as the new base on hit-rate.


---

- finished: 2026-10-29T14:22:28+00:00
- duration: 71.85s
- status: ok
- input_tokens: 16
- output_tokens: 6208
- cache_read_tokens: 211549
- cache_write_tokens: 27811
- cost_usd: 0.389022
