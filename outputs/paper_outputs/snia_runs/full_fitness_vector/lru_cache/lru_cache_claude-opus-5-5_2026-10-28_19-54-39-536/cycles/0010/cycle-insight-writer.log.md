# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010
- allow_edits: True
- started: 2026-10-29T08:12:15+00:00

## timeline

### 08:12:19 tool: Bash

```json
{
  "command": "for f in variants/v00{1,2,3}/insight.md hypotheses/H0{1,2,3}.md; do echo \"=== $f\"; cat $f; done",
  "description": "Read variant insights and hypotheses"
}
```

### 08:12:21 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/variants/v001/insight.md"
}
```

### 08:12:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/variants/v002/insight.md"
}
```

### 08:12:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/variants/v003/insight.md"
}
```

### 08:12:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/hypotheses/H01.md"
}
```

### 08:12:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/hypotheses/H02.md"
}
```

### 08:12:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/hypotheses/H03.md"
}
```

### 08:12:28 thinking



### 08:12:39 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/cycle-insight.md",
  "content": "# Cycle 0010 Insight\n\n## Cohort outcome\n\nTwo variants were latency experiments on the old ghost-free cycle-0004 v002 base, and neither changed policy. v001 swapped SipHash for a fixed-seed folded-multiply hasher. v003 split `prev`/`next` into a parallel `Vec<Link>`. v002, on the cycle-0009 v001 base, changed how reprieves are granted: keys that return to the ghost a second time are escalated into protected. All three are Winners. v002 has the only summary hit-rate gain, almost all of it from hm_0 at 16K. v001 is the cleanest latency win: put and both tails improved with bit-identical hit-rate. v003 won only on the get tail.\n\n## Patterns\n\n- Get p50 did not move for any lever again. Cheaper hashing (v001) and denser links (v003) both left the get median flat or slightly worse, and v002's small get-p50 drop has no code cause. Hash cost and neighbour density are now ruled out as limits on the median. What remains is the hit entry's own memory traffic: the map probe and the node line(s). Splitting the node into two lines made the median worse.\n- Get p99 has now been \"fixed\" by two unrelated changes (a hasher swap and a layout split), just as in cycle 7. The parent's get-p99 outlier is most likely seeding/layout noise. Neither tail gain is established.\n- Put cost tracks hash cost, now for the sixth cycle. A cheaper hasher cut put on both hashes of an evicting put. Extra lines touched (v003) or longer escalation paths (v002) raise the put tail.\n- Escalation on a second ghost hit works only where the ghost window is wide enough to see a second return. That holds at full width at 16K and fails at capacity/4 at 131K. Taking a tag bit for the repeat flag cost web_0 at both sizes through more collisions. Collisions in the ghost now have a visible price whenever they carry policy state.\n- The noise-floor measurement asked for in cycle 9 was not run. Every per-op claim this cycle is still from single runs.\n\n## Open questions\n\n- Does the folded-multiply hasher stack cleanly onto the cycle-0009 v001/cycle-0010 v002 lineage, which already uses a zero-seeded ghost hash?\n- Can the repeat signal be kept without giving up a tag bit, for example in a side bit or a separate tiny table, to remove the web_0 loss?\n- At 131K, is a second-return signal reachable at all with a capacity/4 ghost, or does it need a wider window for returners only?\n- Does a smaller node (not a split one) move get p50?\n\n## Continuation\n\nMake the base cycle-0010 v002 with v001's folded-multiply hasher ported onto it. Recover web_0 by storing the repeat flag without shrinking the ghost tag. Drop the SoA link split and drop hashing as a get-p50 lever. If get p50 is pursued at all, target the hit entry's own line count (node shrinking, probe locality), and only after a paired, repeated noise-floor run exists.\n"
}
```

### 08:12:46 chat

I wrote the Cycle 0010 insight to `cycles/0010/cycle-insight.md`.

**What happened this cycle:**
- **v002 (hit-rate):** Keys that hit the ghost a second time are now moved into the protected segment. This gave the only gain in summary hit-rate, almost all of it on hm_0 at 16K. It also lost slightly on web_0 at both sizes, because the repeat flag uses up a tag bit and causes more ghost collisions.
- **v001 (latency):** Replacing SipHash with a cheaper hasher was the cleanest latency win. Put and both tail latencies improved, and hit-rate stayed bit-identical.
- **v003 (layout):** Moving the `prev`/`next` links into a separate array helped only the get tail. Get p50 got worse.
- v001 and v003 both started from the older cycle-0004 v002 base, not from cycle 9's.

**Patterns I recorded:**
- Get p50 still doesn't move. Hashing and link density are now ruled out as what limits it.
- Get p99 was "fixed" again by two unrelated changes, as in cycle 7. That points to noise.
- Put cost follows hash cost for the sixth cycle in a row.
- The second-ghost-hit rule only pays off where the ghost is wide enough to see a key come back twice: full width at 16K, but not the capacity/4 ghost at 131K.
- Cycle 9 asked for a measured noise floor, but none was run. Every timing claim this cycle is still from single runs.

**Continuation for cycle 11:**
- Use v002 as the base and port v001's hasher onto it.
- Store the repeat flag without shrinking the ghost tag, to win back the web_0 loss.
- Drop the link split, and drop hashing as a way to improve get p50.
- Only go after get p50 again once a paired, repeated noise-floor run exists, and then aim at how many cache lines the hit entry itself touches.


---

- finished: 2026-10-29T08:12:47+00:00
- duration: 32.22s
- status: ok
- input_tokens: 8
- output_tokens: 2637
- cache_read_tokens: 91636
- cache_write_tokens: 16846
- cost_usd: 0.205867
