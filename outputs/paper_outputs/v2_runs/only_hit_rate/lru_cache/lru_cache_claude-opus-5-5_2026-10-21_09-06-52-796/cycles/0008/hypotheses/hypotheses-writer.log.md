# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0008/hypotheses
- allow_edits: True
- started: 2026-10-21T09:47:13+00:00

## timeline

### 09:47:17 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796; cat cross-cycle-insights.md leaderboard.md cycles/0007/cycle-insight.md cycles/0007/cycle-winners.json; tail -c 3000 cycles-ledger.jsonl; ls cycles/0007/variants/*/attempts; cat cycles/0007/variants/*/insight.md"
}
```

### 09:47:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cross-cycle-insights.md"
}
```

### 09:47:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/leaderboard.md"
}
```

### 09:47:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/cycle-insight.md"
}
```

### 09:47:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles-ledger.jsonl"
}
```

### 09:47:26 chat

Now the winner's code.

### 09:47:27 tool: Glob

```json
{
  "pattern": "cycles/0007/variants/v002/attempts/*/overlay/**",
  "path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796"
}
```

### 09:47:27 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/inputs/subjects/lru_cache/baseline"
}
```

### 09:47:29 thinking



### 09:47:29 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v002/attempts"
}
```

### 09:47:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v002/insight.md"
}
```

### 09:47:32 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/variants/v002/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 09:47:57 thinking



### 09:48:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0008/hypotheses/H01.md",
  "content": "# H01\n\nCarry surplus resident-hit credits from `small` into `main` on promotion, together with `MAX_FREQ = 7`, to raise hit rate by letting multi-hit keys in `main` outlast keys that showed reuse only once.\n\n## Parent\n\ncycle/0007/v002\n\n## Changes\n\nAll edits are in `src/lru_cache/mod.rs`.\n\n1. Change `const MAX_FREQ: u8 = 3;` to `const MAX_FREQ: u8 = 7;`.\n2. In `evict_one()`, `small` branch: when the popped node has `node.freq > 0`, it is currently reset with `node.freq = 0` before `self.main.push_back(idx)`. Replace that reset with `node.freq -= 1;`. The first resident hit pays for admission to `main`, as it does now. Every extra hit seen while in `small` becomes a carried credit, so a key hit once in `small` still arrives with `freq = 0` and keeps today's behaviour. Keep `node.in_main = true` and the `continue`.\n3. Leave the ghost-hit path in `put()` unchanged. Ghost arrivals still start at `freq = 0`. This keeps one rule for both admission paths: a key's starting credit in `main` is the number of resident hits it had beyond the first, and a ghost arrival had none. Unlike cycle 7's v003, ghost arrivals get no free credit.\n4. Leave `get`, the existing-key branch of `put`, the `main` branch of `evict_one` (decrement and reinsert), `push_ghost`, `small_target = capacity / 40` and `ghost_cap = 3 * capacity / 20` unchanged.\n5. Termination still holds: each loop iteration either lowers some node's `freq` by at least 1 or frees a slot.\n6. Tests: add an in-file test. Put a key in `small`, `get` it three times, then force it out of `small`. Assert it is now `in_main` and its node has `freq == 2`. Also check that a key with a single hit arrives with `freq == 0`. The existing tests must still pass.\n\n## Rationale\n\nCycle 7 found that a higher `MAX_FREQ` barely helps because both admission paths reset credits to zero. Credits then only build up during `main` residency, which with lazy reinsertion is slow. Carrying credits over is the evidence-respecting way to make the higher cap matter. The credit comes only from hits actually observed while resident, which is exactly the stronger evidence that v003's ghost credit lacked. If hit rate rises over v002 by more than `MAX_FREQ = 7` alone gave (cycle 7 v001), carryover is the lever. If it is flat, credit work in `main` is exhausted.\n"
}
```

### 09:48:18 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0008/hypotheses/H02.md",
  "content": "# H02\n\nReplace `main`'s FIFO-with-reinsertion with a SIEVE-style in-place hand, so survivors keep their position and new arrivals are examined first, to raise hit rate through quicker demotion of keys that entered `main` and were never reused.\n\n## Parent\n\ncycle/0007/v002\n\n## Changes\n\nAll edits are in `src/lru_cache/mod.rs`. `small`, the ghost, the split constants (`capacity / 40`, `3 * capacity / 20`), `MAX_FREQ = 3`, `get` and both admission routes into `main` stay exactly as in v002. Only how `main` picks its victim changes.\n\n1. Make `main` an intrusive doubly linked list over the arena. Add `prev: u32` and `next: u32` to `Node` (sentinel `NIL = u32::MAX`). Replace `main: VecDeque<u32>` with `main_head: u32` (newest), `main_tail: u32` (oldest), `main_len: usize` and `hand: u32` (`NIL` = start at the tail). Add the helpers `main_push_head(idx)` and `main_unlink(idx)`. If the unlinked node is the one under `hand`, `main_unlink` must first move `hand` to that node's `prev`.\n2. Every place that did `self.main.push_back(idx)` for a new arrival must call `main_push_head(idx)` instead. That covers promotion from `small` in `evict_one()` and a ghost hit in `put()`.\n3. Rewrite the `main` branch of `evict_one()` as a SIEVE sweep. Start at `hand`, or at `main_tail` if `hand == NIL`. While the current node has `freq > 0`, decrement its `freq` and leave it **in place**. Do not unlink or reinsert it. Then step `hand` to `node.prev` (toward the head), wrapping to `main_tail` when it reaches `NIL`. When a node with `freq == 0` is found, set `hand = node.prev`, unlink the node, remove its key from `map` and return its slot. Main evictions still do not go into the ghost.\n4. Replace `self.main.is_empty()` / `self.main.len()` with `main_len == 0` / `main_len`. Update the in-file tests that assert `small.len() + main.len() == map.len()` to use `main_len`. Keep that invariant, and add a debug check that walking from `main_head` reaches exactly `main_len` nodes.\n5. Termination: every step of the sweep either decrements a positive `freq` or evicts a node, so the loop is bounded by `main_len * MAX_FREQ + 1` steps.\n6. Tests: add a test in which an old `main` survivor (freq > 0) is retained, and a newly promoted key with `freq == 0` that sits between the hand and the head is evicted on a later sweep before the survivor is visited again. Also run the existing churn tests with the new invariant.\n\n## Rationale\n\nEvery win so far has come from admission filtering, and retention tuning in the form of credit caps has stalled. This is a different retention lever: the order in which `main` is scanned. In FIFO with reinsertion, a surviving key moves behind every new arrival, so a new arrival that is never reused sits in `main` for a full rotation. In SIEVE, survivors stay where they are and the hand reaches recent arrivals sooner relative to proven keys. With `small` now only a fortieth of capacity, more keys arrive in `main` with weak evidence (ghost hits with `freq = 0`), so demoting them quickly should matter more than it did before. If this does not beat v002's 0.7377, scan order is not a limiting factor.\n"
}
```

### 09:48:27 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0008/hypotheses/H03.md",
  "content": "# H03\n\nHalve `small` again to `capacity / 80` and grow the ghost to `13 * capacity / 80` so the combined admission window stays the same, to test whether shifting more resident probation into the ghost still raises hit rate or whether `capacity / 40` is the floor.\n\n## Parent\n\ncycle/0007/v002\n\n## Changes\n\nAll edits are in `src/lru_cache/mod.rs`, in `try_new()`. This is a single-knob change.\n\n1. Change `let small_target = std::cmp::max(1, capacity / 40);` to `std::cmp::max(1, capacity / 80);`. At size 2048 this gives 25, down from 51.\n2. Change `let ghost_cap = std::cmp::max(1, capacity.saturating_mul(3) / 20);` to `std::cmp::max(1, capacity.saturating_mul(13) / 80);`. At size 2048 this gives 332, up from 307. `small + ghost` becomes 357, against v002's 358, so the combined window stays essentially the same.\n3. Update the `log::info!` line and the comments to say `capacity / 80` and `13 * capacity / 80`.\n4. Do not change `MAX_FREQ` (it stays 3), `get`, `put`, `evict_one` or `push_ghost`. The split is the only variable, so the result can be compared directly with the cycle 6 → 7 step.\n5. Update the in-file tests that pin ratios. In `split_ratios_at_2048`, expect `small_target == 25`, `ghost_cap == 332` and a sum of 357. In `capacity_bound_under_churn_400`, expect `small_target == 5` and `ghost_cap == 65`. In `ghost_hit_admitted_into_main` and `ghost_window_expires_after_ghost_cap_plus_one_evictions` at capacity 16, `ghost_cap` becomes `208 / 80 = 2`, unchanged, and `small_target` stays 1, so their asserts still hold. Re-check every hard-coded constant anyway. Add a churn test at capacity 1600 (`small_target = 20`) so the new ratio is exercised with a non-trivial `small`.\n\n## Rationale\n\nThe halving steps from `/10` to `/20` to `/40` each helped, by less each time. That is the same shrinking-gains curve the ghost-only halvings followed before they went flat at `/16`. This probe finds where the curve turns. If hit rate rises, the ghost is still cheaper probation than resident slots. If it falls, the two forces cycle 7 named have taken over: less reuse caught in `small`, and more ghost arrivals with no credit. Either way the result settles the `small`/ghost split and frees later cycles for other levers. Compare against H01: if carrying credits over helps, a smaller `small` may need it, and the next cycle could combine the two.\n"
}
```

### 09:48:35 thinking



### 09:48:35 chat

I've written three hypotheses for cycle 0008 into the `hypotheses` directory as `H01.md` to `H03.md`. All three use the current best, `cycle/0007/v002` (hit rate 0.7377), as their parent, and each tries a different lever:

- **H01: carry credits from `small` into `main`.** When a key is promoted from `small`, it keeps `freq - 1` instead of being reset to 0. A key hit once in `small` therefore arrives in `main` exactly as it does now. Ghost arrivals still start at 0, so both admission paths follow one rule: a key's starting credit is its resident hits beyond the first. `MAX_FREQ` goes to 7. This tests cycle 7's open question of whether a higher frequency cap pays off once credits are no longer reset when a key moves to `main`.
- **H02: change how `main` scans for victims.** `main` becomes a SIEVE-style list: keys that survive a pass keep their position, new arrivals go in at the head, and an in-place hand does the eviction. Admission and all the constants stay unchanged. The idea is that keys entering `main` and never reused get evicted sooner. This is a retention lever no previous cycle has tried.
- **H03: find the floor for the `small`/ghost split.** `small` drops from `capacity / 40` to `capacity / 80`, and the ghost grows to `13 * capacity / 80`. That keeps the combined window at about 357 slots at cache size 2048, against 358 now, so the split is the only thing that changes. The in-file tests that pin the ratios are updated, and a churn test at capacity 1600 is added.

Two gaps from earlier cycles are still open. No hypothesis covers testing a second cache size or measuring `metadata-size` and `per-op-time`, because those need harness changes outside what variants are allowed to edit. The fingerprint-ring port was also left out of this cohort.

I couldn't list directories in this session, so I read only the winner's code (`v002`). I didn't read the overlays of cycle 7's v001 and v003, or the baseline source. The cycle summaries describe what those variants changed, which was enough to keep these three hypotheses from repeating them.


---

- finished: 2026-10-21T09:48:36+00:00
- duration: 83.07s
- status: ok
- input_tokens: 14
- output_tokens: 7453
- cache_read_tokens: 196413
- cache_write_tokens: 31045
- cost_usd: 0.436759
