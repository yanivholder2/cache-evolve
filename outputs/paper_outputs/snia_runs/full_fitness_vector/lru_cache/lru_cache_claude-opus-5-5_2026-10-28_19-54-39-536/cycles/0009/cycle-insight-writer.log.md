# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0009
- allow_edits: True
- started: 2026-10-29T08:03:27+00:00

## timeline

### 08:03:31 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0009; for f in variants/v00{1,2,3}/insight.md hypotheses/H0{1,2,3}.md; do echo \"=== $f\"; cat $f; done",
  "description": "Read variant insights and hypotheses"
}
```

### 08:03:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0009/variants/v001/insight.md"
}
```

### 08:03:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0009/variants/v002/insight.md"
}
```

### 08:03:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0009/variants/v003/insight.md"
}
```

### 08:03:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0009/hypotheses/H01.md"
}
```

### 08:03:36 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0009/hypotheses/H02.md"
}
```

### 08:03:36 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0009/hypotheses/H03.md"
}
```

### 08:03:42 thinking



### 08:03:55 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0009/cycle-insight.md",
  "content": "# Cycle 0009 Insight\n\n## Cohort outcome\n\nTwo variants made the ghost smaller and one replaced it. v001 stacked capacity/4 with 8-bit tags. v003 replaced cycle-0008 v002's tag table with a 1-bit bitset aged by a rotating cursor. v002 dropped the ghost and gave demoted-from-protected keys a reprieve marked by a flag bit. All three are Winners, but only v001 is a clean gain: get and put were both faster, at a cost of the smallest visible hit-rate loss. v003 kept hit-rate roughly level but made get p50 and put p50 worse. v002 lost hit-rate and did not move latency in any interpretable way.\n\n## Patterns\n\n- The ghost-footprint theory of get p50 is now in doubt. At 131K, v001 and v003 have the same ghost bytes, and get never touches the ghost in either. Yet v001's get p50 fell and v003's rose. v003's ghost is even smaller at 16K. So footprint alone cannot explain the get-p50 moves. Noise, or layout or codegen effects from changes to the header and to `put`, are at least as likely. Cycle 8's \"isolated cause\" should be downgraded to \"unconfirmed\".\n- Get p50 again moved without any change to the get path. v002 rose with no new get branch, and v003 rose with byte-identical get code. This is the same noise or layout sensitivity seen in cycles 6–8. No single-run get-p50 delta of this size counts as evidence.\n- Put cost tracks work in the `Vacant` arm. A denser table made put cheaper (v001). Cursor bookkeeping and read-modify-write bit operations made it more expensive (v003), even with fewer bytes. This continues the multi-cycle \"put cost = hash/bookkeeping count\" pattern.\n- Ghost collisions now show up in hit-rate. v001 (8-bit tags) and v003 (tagless bits) both moved hit-rate on real traces. So cycle 8's bit-identical result was luck at that configuration, not proof that the harness never runs the ghost path. The hit-rate cost of shrinking the ghost is real but marginal at both sizes.\n- Reprieves need evidence gathered after eviction. A flag for \"was once protected\" (v002) predicts reuse poorly: it keeps cold keys at the expense of newcomers, especially in the small probation at 16K. Only a ghost hit, meaning a re-reference after eviction, has paid off in any cycle.\n\n## Open questions\n\n- Is v001's get-p50 gain larger than the noise? Run paired, repeated measurements of v001, its parent, and v003 to decide whether footprint matters at all.\n- At 131K, does the ghost still pay for its bytes and put cost compared with no ghost at all, given its short window there?\n- Can the bitset ghost keep its hit-rate without paying the per-put aging cost, for example with a generation flip on a double buffer instead of a word sweep?\n\n## Continuation\n\nUse cycle-0009 v001 (capacity/4, 8-bit tags) as the base. Before claiming any more get-p50 effects, establish a measured noise floor using paired repeated runs. Drop flag-only or pre-eviction-history reprieves. Drop further ghost shrinking as a get-latency lever: judge ghost changes only on hit-rate and put cost. If a bitset is retried, aging must add no per-put work. Any new hit-rate lever should change how ghost-hit reprieves are granted, not the window size.\n"
}
```

### 08:04:01 chat

I've written `cycle-insight.md` for cycle 0009. The main finding is that cycle 8's explanation for get p50 (the ghost table's byte footprint) is now unconfirmed. At 131K, v001 and v003 have the same ghost bytes, and neither touches the ghost on get. Yet v001's get p50 fell and v003's rose.

- **v001 (capacity/4 ghost with 8-bit tags):** the only clean gain. Get and put both got faster, and hit-rate dropped by the smallest step the metric shows.
- **v003 (1-bit bitset ghost aged by a rotating cursor):** hit-rate stayed roughly level, but get p50 and put p50 both rose. Put got slower because of the extra aging work in the eviction path.
- **v002 (no ghost; demoted keys get one reprieve via a flag bit):** hit-rate dropped. A key's earlier time in protected turned out to be a poor predictor of reuse. Only re-references after eviction, which the ghost catches, have paid off in any cycle.

**Other patterns:**
- Get p50 moved again on code where the get path hadn't changed. Single-run get-p50 deltas of this size can't count as evidence.
- Both shrunken ghosts lost a little hit-rate on the real traces. Cycle 8's identical hit-rate with 8-bit tags was luck, not proof that tag collisions never happen in the harness.

**Continuation for the next cycle:**
- Base on cycle-0009 v001.
- Measure the noise floor with repeated paired runs before making any more get-p50 claims.
- Drop flag-only reprieves.
- Stop treating ghost shrinking as a way to speed up get; judge ghost changes only on hit-rate and put cost.
- If a bitset ghost is tried again, its aging must add no work to each put.


---

- finished: 2026-10-29T08:04:02+00:00
- duration: 34.96s
- status: ok
- input_tokens: 8
- output_tokens: 3005
- cache_read_tokens: 90509
- cache_write_tokens: 17572
- cost_usd: 0.21881
