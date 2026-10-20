# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007
- allow_edits: True
- started: 2026-10-19T14:19:23+00:00

## timeline

### 14:19:26 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007; for f in variants/v00*/insight.md hypotheses/H0*.md; do echo \"=== $f\"; cat \"$f\"; done",
  "description": "Read variant insights and hypotheses"
}
```

### 14:19:28 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/variants/v001/insight.md"
}
```

### 14:19:29 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/variants/v002/insight.md"
}
```

### 14:19:29 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/variants/v003/insight.md"
}
```

### 14:19:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/hypotheses/H01.md"
}
```

### 14:19:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/hypotheses/H02.md"
}
```

### 14:19:31 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/hypotheses/H03.md"
}
```

### 14:19:36 thinking



### 14:19:46 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/cycle-insight.md",
  "content": "# Cycle 0007 Insight\n\n## Cohort outcome\n\nAll three hypotheses started from cycle 0006 v001 (16-byte item, 7/8 cap), and each changed one separate axis:\n- v001 finally merged cycle 0005's tag-first single-hash `find`.\n- v002 limited protected-hit promotion to move-ahead-4.\n- v003 raised the cap to 15/16 on buckets of 16+ slots.\n\nv001 is the clear Winner: get and put both got faster, and hit rate and metadata did not change. v003 gained a small amount of hit rate at no latency cost. v002 kept hit rate but got slower on get and much slower on put. It is a Winner by default, not on merit.\n\n## Patterns\n\n- Lookup and replacement can be tuned separately. Tag-first lookup with one hash cut latency without changing any replacement decision. Put gained more than expected, because the ghost and eviction paths now reuse stored tags instead of rehashing. Keeping the tag array in sync costs much less than the key compares and hashes it removes. That answers the open question about tag upkeep.\n- Long protected rotates are not what makes the 7/8 cap slow. Protected segments are at most 14–15 items of 16 bytes each, so even a full rotate is cheap. Bounding it gained nothing and added variable-length moves. Both v002 and v003 point the other way: the cost comes from the thin probation segment. A 1-slot probation means more evictions, ghost writes and ghost scans, and more whole-bucket insert/remove shifts on put.\n- The protected-cap lever is nearly used up. 15/16 still adds hit rate on large buckets, but much less than the 3/4 → 7/8 step did. Its cost was low only because it reuses the existing in-place replace path. Probation now holds cold keys for only one miss.\n- Approximate recency inside protected hurt indirectly. It demoted still-warm keys into probation, and that churn showed up on the put tail even though put's own code got lighter.\n\n## Open questions\n\n- With probation down to one slot, how much of the median put cost on a miss comes from the linear ghost scan and ghost writes? Would a hashed or bitset ghost lookup remove it?\n- Can admission decide more of the hit rate now that the cap is flattening? Examples: a frequency or ghost-based filter that keeps one-hit-wonders out of probation altogether.\n- Do whole-bucket insert/remove shifts in probation eviction dominate the put tail? Would a ring-offset layout for probation alone remove them?\n\n## Continuation\n\nBuild the base from v001 (tag-first find, single hash, tags reused on ghost and evict) plus v003's 15/16 cap. Spend latency work on the miss and eviction path: non-linear ghost lookup, and cheaper probation insert/evict shifts. Leave protected-side rotates alone. Look for further hit-rate gains in admission or ghost policy, not in a larger protected share. Drop bounded or approximate promotion inside protected, and stop pushing the cap past 15/16.\n"
}
```

### 14:19:52 chat

I wrote `cycle-insight.md` in the cycle 0007 output dir. All three variants started from cycle 0006 v001 (the 16-byte item with the 7/8 protected cap), and each changed one thing.

**Cohort outcome**
- **v001 is the clear Winner.** It added the tag-first, single-hash lookup from cycle 0005. Get and put both got faster, and hit rate and metadata size stayed the same. Put gained more than expected because the ghost and eviction paths now reuse the stored tags instead of hashing again. Keeping the tag array in sync costs much less than the work it removes.
- **v003 gained a little.** It raised the protected cap to 15/16 on buckets of 16 or more slots. Hit rate went up slightly with no latency cost, but the gain was much smaller than the earlier step from 3/4 to 7/8, so the cap is nearly used up.
- **v002 is a Winner by default, not on merit.** It limited how far a protected hit moves toward the front (4 slots). Hit rate held, but get got slower and put got much slower.

**Main pattern:** v002 shows that long moves inside protected are not what makes the 7/8 cap slow. Those moves are already cheap at these bucket sizes. Both v002 and v003 point to the one-slot probation segment instead. It forces frequent evictions, ghost writes and ghost scans, and whole-bucket shifts on put.

**Continuation for the next cycle:**
- Build the base from v001 plus v003's 15/16 cap.
- Aim latency work at the miss and eviction path: a ghost lookup that doesn't scan the ring, and cheaper probation insert and evict shifts.
- Look for more hit rate in admission or ghost policy, not a bigger protected segment.
- Drop approximate promotion inside protected, and stop raising the cap past 15/16.


---

- finished: 2026-10-19T14:19:53+00:00
- duration: 29.97s
- status: ok
- input_tokens: 8
- output_tokens: 2631
- cache_read_tokens: 88655
- cache_write_tokens: 16605
- cost_usd: 0.203223
