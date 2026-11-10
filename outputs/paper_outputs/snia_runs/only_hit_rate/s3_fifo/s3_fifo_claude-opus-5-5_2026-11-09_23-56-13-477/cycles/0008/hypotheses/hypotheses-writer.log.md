# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0008/hypotheses
- allow_edits: True
- started: 2026-11-10T10:13:36+00:00

## timeline

### 10:13:39 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477; cat leaderboard.md cross-cycle-insights.md cycles/0007/cycle-insight.md; tail -c 6000 cycles-ledger.jsonl; ls cycles/0007/variants/*/attempts; cat cycles/0007/variants/*/insight.md"
}
```

### 10:13:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cross-cycle-insights.md"
}
```

### 10:13:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/leaderboard.md"
}
```

### 10:13:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0007/cycle-insight.md"
}
```

### 10:13:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles-ledger.jsonl"
}
```

### 10:13:49 chat

Now the v001 overlay of cycle 0007.

### 10:13:49 tool: Glob

```json
{
  "pattern": "cycles/0007/variants/v001/**",
  "path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477"
}
```

### 10:13:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0007/variants/v001/attempts/01/overlay/src/s3_fifo/mod.rs"
}
```

### 10:13:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0007/cycle-winners.json"
}
```

### 10:14:09 thinking



### 10:14:20 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0008/hypotheses/H01.md",
  "content": "# H01\n\nMoving the ghosts of hit main victims into their own FIFO, with its own budget and without advancing the shared ghost stamp clock, will keep v001's large-size hit-rate gains and remove its web_0 small-size loss.\n\n## Parent\n\ncycle/0007/v001\n\n## Changes\n\nSplit the ghost into two independent structures in `src/s3_fifo/mod.rs`:\n\n- Keep the existing `ghost: VecDeque<(K, u64)>`, `ghost_live`, `next_stamp` and the `main_size` bound for **small-queue victims only**. `evict_small` keeps calling `ghost_insert(key, freq)` unchanged, so the freq-0 recency window (`next_stamp - stamp <= small_size`) again counts only small→ghost insertions. This is exactly the cycle 0005 v002 clock.\n- Add a second FIFO `main_ghost: VecDeque<(K, u64)>` with its own `main_ghost_live: usize` and its own counter `main_ghost_stamp: u64`. Its budget is `MAIN_GHOST_BUDGET = max(1, small_size)`: one small-queue's worth of keys, which is a small fraction of the main ghost. Add a new `Loc` variant `Loc::MainGhost { stamp: u64 }` (stamped with `main_ghost_stamp`), or reuse `Loc::Ghost` with a 1-bit tag. Pick whichever keeps `Loc` at 16 bytes or less.\n- In `evict_main`, a victim with `main_hit == true` calls a new `main_ghost_insert(key)` instead of `ghost_insert(key, GHOST_TO_MAIN_FREQ)`. It pops stale or overflow entries from `main_ghost`'s back, using its own stamp match, and lazily compacts once `main_ghost.len() > 2 * budget`. It must not touch `next_stamp`, `ghost_live` or `ghost`.\n- In `put`, add a match arm for `Loc::MainGhost { .. }`. Remove the index entry, decrement `main_ghost_live`, `evict_main()` if main is full, then push the key to main's front with freq 0 and `main_hit = false`, as the strong-returnee arm does now. Keep the \"remove the index entry before evicting\" ordering for re-entrancy, because `evict_main` can itself insert into `main_ghost`.\n- `get` must return `None` for `MainGhost`.\n- Invariants: `index.len() == resident + ghost_live + main_ghost_live`; `ghost_live <= main_size`; `main_ghost_live <= budget`.\n- Update the in-file `Reference` oracle so that it keeps a separate `main_ghost` list with the same budget, and so that its `ghost_push` stamps advance only for small victims. Adjust the unit tests for main-victim ghosts to assert `MainGhost`.\n\n## Rationale (optional)\n\nCycle 0007 found that v001's small-size loss came from main-victim ghosts taking up the shared ghost budget and advancing the stamps. This shrank the freq-0 age window, which cycles 6 and 7 showed is a local optimum at 1×. Decoupling restores the parent's window exactly while keeping the only mechanism so far that moved web_0 at the large size. The experiment is direct: compare this variant's four cells against v001 and cycle 0005 v002.\n"
}
```

### 10:14:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0008/hypotheses/H02.md",
  "content": "# H02\n\nShrinking the small queue to 10% of capacity only at large cache sizes, while pinning the ghost-age window to its old 25% value, will raise web_0 hit-rate at the large size without disturbing the small-size cells.\n\n## Parent\n\ncycle/0007/v001\n\n## Changes\n\nIn `S3Fifo::try_new` in `src/s3_fifo/mod.rs`, make the split depend on size:\n\n- Add `const LARGE_CAPACITY: usize = 65_536;` and `const LARGE_SMALL_DIVISOR: usize = 10;`.\n- If `capacity >= LARGE_CAPACITY`, set `small_size = (capacity / LARGE_SMALL_DIVISOR).max(1).min(capacity - 1)`. Otherwise keep the existing `(capacity / 4).max(1).min(capacity - 1)`. `main_size = capacity - small_size` as before. The ghost bound (`main_size`) and compaction bound follow automatically.\n- Add a new struct field `recent_window: u64`. Compute it once in `try_new` as `((capacity / 4).max(1).min(capacity - 1) / GHOST_RECENT_AGE_DIVISOR).max(1)`, i.e. the window the parent would have had. Change `ghost_recent_window()` to return `self.recent_window`. This isolates the split change from the age-gate window, which is a confirmed 1× local optimum.\n- Leave all routing, graduation, main-hit ghosting and the freq reset unchanged.\n- Update the `Reference` oracle in the same way: the size-dependent `small_size`, plus a stored `recent` computed from the 25% value. Add a unit test checking that `capacity = 131072` gives `small_size = 13107` and `recent_window = 32768`, and that `capacity = 16384` is unchanged.\n\n## Rationale (optional)\n\nweb_0 at the large size has resisted every ghost and small routing change. The small/main split at that size is the one lever named as unprobed in the cross-cycle insights. Earlier split tweaks applied to all sizes, and they also moved the age window, which scales with `small_size`. This variant changes only the large-size split, so if the small-size cells move by even one ulp, that points to an implementation error. Any change in the large-size cells can be attributed to the split, since the window is held fixed.\n"
}
```

### 10:14:36 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0008/hypotheses/H03.md",
  "content": "# H03\n\nDoubling the ghost budget from `main_size` to `2 * main_size`, with the age-gate stamp clock unchanged, will raise hit-rate by giving more long-gap returnees the freq-1 small path instead of being treated as never-seen first admissions.\n\n## Parent\n\ncycle/0005/v002\n\n## Changes\n\nIn `src/s3_fifo/mod.rs`, change only the ghost capacity:\n\n- Add `const GHOST_BUDGET_MULT: usize = 2;` and a field (or helper) `ghost_cap = (GHOST_BUDGET_MULT * main_size).max(1)`.\n- In `ghost_insert`, change the live-ghost eviction check from `self.ghost_live >= self.main_size` to `self.ghost_live >= ghost_cap`. Change the lazy compaction trigger from `ghost.len() > 2 * main_size` to `ghost.len() > 2 * ghost_cap`, and size the initial `ghost` deque and `index` capacities to match.\n- Do **not** change how `next_stamp` advances or how the recency window is computed (`small_size / GHOST_RECENT_AGE_DIVISOR`). The age gate measures ghost insertions, not ghost occupancy, so the quick freq-0 → main path keeps exactly the same population. Only keys whose ghost record would previously have expired gain from the change:\n  - freq-0 keys go back to small with freq 1;\n  - freq ≥ 1 keys go straight to main.\n- Leave graduation, freq reset and main eviction as in the parent. Do not add main-victim ghosting.\n- Update the `Reference` oracle's `ghost_push` bound to `2 * main_size`. Extend `ghost_compaction_bound_holds` to assert `ghost_live <= ghost_cap` and `ghost.len() <= 2 * ghost_cap`.\n\n## Rationale (optional)\n\nThe ghost's size has never been varied in seven cycles. It is the remaining first-admission lever. Today a key that returns after more than `main_size` ghost insertions is indistinguishable from a never-seen key. A larger ghost separates \"seen once long ago\" from \"brand new\" without touching the tuned age window or the shared clock. Those are the two couplings that broke the cycle 6 and 7 ghost-path variants. The 2× ghost costs extra metadata, but metadata-size is not currently scored. If web_0 punishes late rereferences again, the result should show it cleanly in the large-size web_0 cell.\n"
}
```

### 10:14:46 thinking



### 10:14:46 chat

I've written three hypotheses for cycle 0008 to `cycles/0008/hypotheses/`. Two of them build on the current best variant (cycle/0007/v001, hit-rate 0.54865). Each one tests a lever that hasn't been tried yet.

| File | Parent | Change being tested |
| --- | --- | --- |
| `H01.md` | cycle/0007/v001 | Main-queue victims that were hit get their own ghost list, sized to the small queue. Inserting into it doesn't advance the shared ghost clock. The goal is to keep v001's gains at the large cache size without its loss on web_0 at the small size. |
| `H02.md` | cycle/0007/v001 | At caches of 65,536 entries or more, the small queue drops to 10% of capacity. The ghost-age window stays at its old 25% value, so only the split changes. This is aimed at web_0 at the large size, which no change has moved meaningfully yet. |
| `H03.md` | cycle/0005/v002 | The ghost list doubles to twice the main queue's size, and the age gate is left alone. Keys that come back after a long gap then take the freq-1 route through the small queue instead of being treated as brand-new keys. |

How they relate to cycle 0007's findings:
- **H01** is the one decoupled follow-up to v001 that cycle 0007 asked for. It tests the explanation that v001's small-size loss came from sharing the ghost list and its clock.
- **H02** is the first split change limited to the large size. If the small-size results change at all, that points to an implementation error, not to the policy.
- **H03** varies ghost size, which hasn't been changed in seven cycles. It costs extra metadata, but metadata-size still isn't in the scores.

Every hypothesis also asks for the built-in reference model and its unit tests to be updated, so the check that the fast and reference versions give the same hits and misses still applies.


---

- finished: 2026-11-10T10:14:47+00:00
- duration: 71.42s
- status: ok
- input_tokens: 12
- output_tokens: 6120
- cache_read_tokens: 170615
- cache_write_tokens: 36409
- cost_usd: 0.448094
