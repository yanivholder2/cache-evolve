# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0004
- allow_edits: True
- started: 2026-11-05T22:46:15+00:00

## timeline

### 22:46:20 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0004; for f in variants/v00*/insight.md hypotheses/H0*.md; do echo \"=== $f\"; cat \"$f\"; done",
  "description": "Read variant insights and hypotheses"
}
```

### 22:46:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0004/variants/v001/insight.md"
}
```

### 22:46:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0004/variants/v002/insight.md"
}
```

### 22:46:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0004/variants/v003/insight.md"
}
```

### 22:46:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0004/hypotheses/H01.md"
}
```

### 22:46:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0004/hypotheses/H02.md"
}
```

### 22:46:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0004/hypotheses/H03.md"
}
```

### 22:46:38 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0004/cycle-insight.md",
  "content": "# Cycle 0004 Insight\n\n## Cohort outcome\n\nAll three variants were evaluated, and each one tried to remove or avoid a demotion cascade. Two built on the cycle 0003 v002 line. v001 deferred the second-hit promotion when both normal buckets are full. v002 promoted target-hottest items into probing only when probing has room. The third, v003, added probing overflow to the ungated MRU line. Each variant gained a little hit rate, and each made at least one tail metric worse. There are no Winners.\n\n## Patterns\n\n- Cascade count is not the cost to minimize. Stash fill level and where the cascade cost lands matter more. Skipping a cascade without giving items another way out of stash (v001) kept deferred items in stash permanently, and `put` p99 got much worse. Skipping cascades elsewhere moved their cost rather than removing it: to `put` through fuller target buckets (v002), or to a later `get` through foreign residents in probing (v003). Cycle 0003's overflow helped because it cut cascades and also kept items flowing out of stash.\n- \"Don't evict a proven resident\" works for hit rate. The probing room gate gained a little everywhere, most at the small size. Deferral and ungated overflow gained retention at the small size or held it, but deferral slipped slightly on hm_0 at the large size, which is again the cell that exposes retention loss.\n- `get` p50 did not respond to anything that skips cascades. The remaining median gap to baseline is on the common hit path: the multi-bucket scans and stash checks. It is not in the rarer cascade paths.\n- Items placed in probing pay a lookup cost: they are found last and can be evicted by a neighbour's promotion. On the ungated line this cost more `get` p99 than overflow saved on `put`, so the ungated line is not the cheaper route.\n\n## Open questions\n\n- Can a deferred promotion be bounded, for example by demoting or force-promoting after a limit, so stash stays drained without bringing the full cascade back?\n- Which part of the common hit path holds the `get` p50 gap: the stash check, the scan of a full target, or the cost of touching an item to MRU?\n- Does v002's probing room gate keep its hit-rate gain if the stash-promotion fallback it now triggers more often is made cheaper?\n\n## Continuation\n\nKeep cycle 0003 v002 as the base. v002's probing room gate is a cheap hit-rate gain worth carrying if its `put` cost can be fixed at the stash-promotion fallback. Drop the ungated overflow line and unbounded deferral. Any change that skips promotions must keep items flowing out of stash. Point `get` p50 work at the common hit path (scan cost, stash checks), not at cascades.\n"
}
```

### 22:46:45 thinking



### 22:46:45 chat

I've written the Cycle 0004 insight to `cycle-insight.md` in the cycle's output dir. All three variants were evaluated and there are no Winners: each gained a little hit rate, and each made at least one tail-latency number worse.

- **Main finding:** the number of demotion cascades isn't the real cost. What matters is how full stash gets and where a cascade's cost ends up.
  - **v001** skipped the promotion when target and probing were both full. Twice-hit items then had no way out of stash, so stash filled up and `put` p99 got much worse.
  - **v002** only promoted the hottest target item into probing when probing had room. That moved the cascade cost onto `put`.
  - **v003** added probing overflow to the ungated line. That moved the cost onto a later `get`.
- **Median `get`:** p50 didn't respond to any cascade-skipping change. The remaining gap to baseline is in the normal hit path: scanning several buckets and checking stash.
- **Continuation:**
  - Keep cycle 0003's v002 as the base.
  - Keep this cycle's v002 probing-room rule, which is a small, cheap hit-rate gain, but only if its `put` cost can be fixed at the stash-promotion fallback.
  - Drop the ungated overflow line and promotion deferral that has no limit.
  - Any change that skips promotions must still let items leave stash.
  - Aim the median `get` work at the normal hit path, not at cascades.


---

- finished: 2026-11-05T22:46:45+00:00
- duration: 30.18s
- status: ok
- input_tokens: 8
- output_tokens: 2255
- cache_read_tokens: 88093
- cache_write_tokens: 14639
- cost_usd: 0.180292
