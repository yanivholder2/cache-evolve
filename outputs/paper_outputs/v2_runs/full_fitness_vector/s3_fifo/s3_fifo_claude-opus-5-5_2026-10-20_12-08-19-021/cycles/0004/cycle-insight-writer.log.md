# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004
- allow_edits: True
- started: 2026-10-20T12:37:13+00:00

## timeline

### 12:37:20 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004; for f in variants/v00*/insight.md hypotheses/H0*.md; do echo \"=== $f\"; cat \"$f\"; done",
  "description": "Read variant insights and hypotheses"
}
```

### 12:37:32 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/variants/v001/insight.md"
}
```

### 12:37:32 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/variants/v002/insight.md"
}
```

### 12:37:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/variants/v003/insight.md"
}
```

### 12:37:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/hypotheses/H01.md"
}
```

### 12:37:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/hypotheses/H02.md"
}
```

### 12:37:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/hypotheses/H03.md"
}
```

### 12:37:48 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0004/cycle-insight.md",
  "content": "# Cycle 0004 Insight\n\n## Cohort outcome\n\nAll three variants used cycle 0003's v002 (the fingerprint ring) as their parent, and each was meant to change one lever. H01 halved the ghost ring. H02 froze the split at a static 10/90 with `freq > 1` graduation kept. H03 was a policy-neutral cleanup of the `put` path. All three are Winners, but only v003 (H03) clearly improved: `put` got a little faster and hit-rate was identical. v001 moved hit-rate barely above zero, and v002 lost on both hit-rate and latency, so both made the cut on rank in a small cohort.\n\n## Patterns\n\n- **The adaptive split has now held up four cycles in a row, and it soaks up changes to other levers.** Halving the ghost barely moved hit-rate, because the adaptation and its `capacity/16` / `capacity/2` clamps absorbed most of the change. Freezing the split lost hit-rate even with the incumbent graduation rule kept. Together with cycle 0003's static-split loss, this is the strongest evidence yet that the adaptation itself earns its margin and is not just an escape from a bad starting split.\n- **\"One lever\" variants keep coupling ghost size to the split.** v002 tied `ghost_cap` to the new main size, so the ghost also grew. That repeats the confound from cycle 0003. Any change to the split has to pin ghost size on its own.\n- **`put` gains come from cutting machine work, not hashing.** v003's gain traces to swapping the ring's modulo wrap for a compare and to inlining the eviction functions. Its premise of redundant hashing was a miscount: the parent already hashed and cloned as little as the refactor did. Count hashes and clones in the parent before you claim them as a lever.\n- **`get` latency is noise.** All three variants left `get` byte-identical, and all three saw it move, mostly upward. Because the headline latency metric tracks `get`, a `put`-only refactor cannot improve the summary.\n- **`metadata-size` still misses ghost heap.** It hid both v001's ghost shrink and v002's ghost growth.\n\n## Open questions\n\n- Where does `small_size` actually settle? Four cycles have asked for this, and no variant has reported it.\n- Does a static 25/75 split (the parent's starting point), with the ghost pinned at 75%, match the adaptive split? That would isolate the adaptation from the split ratio and the ghost size.\n- Would loosening or tightening the adaptation clamps turn ghost length back into a lever that matters?\n- Is the tiny gain from a halved ghost real, or within noise?\n- Would a single-probe miss path (a precomputed-hash or raw-entry index) give a real `put` gain beyond v003's?\n\n## Continuation\n\nAdopt v003 as the new parent: it keeps the incumbent policy with a faster `put`. Stop trying to shrink the ghost, and stop replacing the adaptation with any static split unless ghost size is pinned at the parent's value. Productive next levers are the adaptation's clamps and trigger strength, main-ghost reinsertion into `small` (still untested), and a true single-probe index for `put`. Every variant must report where `small_size` settles, and must not credit `get` latency changes when the `get` path is untouched.\n"
}
```

### 12:37:55 thinking



### 12:37:55 chat

I've written `cycle-insight.md` for cycle 0004 in the output dir. All three variants are Winners, but only v003 actually improved on its parent.

- **v003 (H03)** made `put` slightly faster and kept hit-rate exactly the same. The gain came from two small edits: the ghost ring's wrap-around now uses a compare instead of a division, and the eviction functions are inlined. The hypothesis expected savings from hashing less, but the parent already hashed and cloned the minimum, so that part removed nothing.
- **v001 (H01)** halved the ghost ring. Hit-rate rose only very slightly, because the adaptive split and its clamps absorbed most of the change.
- **v002 (H02)** froze the split at a static 10/90. It lost on both hit-rate and latency. The test isn't clean, though: it tied ghost size to the new main size, so the ghost grew too. Cycle 0003 had the same problem.

**Patterns across the cohort:**
- The adaptive split has now beaten every alternative for four cycles running.
- The `get` latency numbers are noise: all three variants left `get` unchanged, yet it moved in every run.
- The `metadata-size` metric still doesn't count the ghost's memory.
- No variant has yet reported where `small_size` settles, despite four cycles of asking.

**Continuation for the next cycle:** use v003 as the parent. Stop shrinking the ghost. Stop trying static splits unless ghost size is held at the parent's value. The levers to try next are:
- the adaptation's clamps and how strongly its triggers fire;
- sending main-ghost hits back into `small` (still untested);
- a `put` path that does a single index lookup per miss.

Every variant must report where `small_size` settles.


---

- finished: 2026-10-20T12:37:56+00:00
- duration: 42.50s
- status: ok
- input_tokens: 8
- output_tokens: 2590
- cache_read_tokens: 85914
- cache_write_tokens: 14884
- cost_usd: 0.188087
