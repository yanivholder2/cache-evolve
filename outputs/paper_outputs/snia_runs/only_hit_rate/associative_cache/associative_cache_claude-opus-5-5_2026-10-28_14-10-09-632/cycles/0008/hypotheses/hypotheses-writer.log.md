# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/hypotheses
- allow_edits: True
- started: 2026-10-28T15:19:53+00:00

## timeline

### 15:19:56 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632; cat cross-cycle-insights.md leaderboard.md cycles/0007/cycle-insight.md; cat cycles-ledger.jsonl | cut -c1-1500; ls cycles/0007/variants; for v in cycles/0007/variants/*; do echo \"== $v\"; cat $v/insight.md; ls $v/attempts; done"
}
```

### 15:19:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cross-cycle-insights.md"
}
```

### 15:19:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/leaderboard.md"
}
```

### 15:19:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles-ledger.jsonl"
}
```

### 15:19:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/cycle-insight.md"
}
```

### 15:19:59 chat

Now the v001 overlay.

### 15:20:00 tool: Glob

```json
{
  "pattern": "cycles/0007/variants/v001/**/*",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632"
}
```

### 15:20:01 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/variants/v001/attempts"
}
```

### 15:20:02 tool: Grep

```json
{
  "pattern": "fn ",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/variants/v001/attempts",
  "output_mode": "files_with_matches"
}
```

### 15:20:04 thinking



### 15:20:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/variants/v001/insight.md"
}
```

### 15:20:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/cycle-winners.json"
}
```

### 15:20:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/variants/v001/attempts/01/overlay"
}
```

### 15:20:07 thinking



### 15:20:07 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/variants/v001/attempts/02/overlay"
}
```

### 15:20:08 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/variants/v001/attempts/01/overlay/src/associative_cache/mod.rs"
}
```

### 15:20:09 thinking



### 15:20:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/hypotheses/H01.md"
}
```

### 15:20:11 thinking



### 15:20:11 chat

I've read the cycle 0007 results and the cross-cycle notes. Next I'll read the winning variant's code, then write the three hypotheses.

### 15:20:11 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0007/variants/v001/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 15:21:04 thinking



### 15:21:19 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/hypotheses/H01.md",
  "content": "# H01\n\nChoosing main's decay rule by main-region size, so small main regions (under 256 slots) go back to linear decrement ranked by `recent + counter` while large ones keep recency-gated halving, raises hit-rate on hm_0 at 16384 without giving up cycle 0007 v001's web_0 gain at 131072.\n\n## Parent\n\ncycle/0007/v001\n\n## Changes\n\nEdit only `src/associative_cache/associative_cache_bucket.rs`, starting from\ncycle/0007/v001. Keep `GCLOCK_MAX = 7`, `PROBATION_FLOOR = 64`, `probation_len`,\nthe probation test (counter 0 at `small_hand` means evict, otherwise promote),\n**reset-to-0 plus recent-bit clear on promotion**, and the recent-bit set on\nevery hit. Do not edit `Item` or anything in `src/shared/`.\n\n1. **Decay-mode field.** Add `const HALVING_MIN_MAIN: usize = 256;` and a field\n   `halving: bool` to `AssociativeCacheBucket`. In `new`, set\n   `halving = max_size - small_len >= HALVING_MIN_MAIN`. For the evaluated\n   sizes, 128-slot buckets (main = 64) get `halving = false` and 1024-slot\n   buckets (main = 768) get `halving = true`. Plain-GCLOCK buckets\n   (`max_size < 4`) get `false`.\n2. **`sweep_visit(h)`.** If `recent[h]`, clear it and return `false` (no\n   change). Otherwise, if `lfu_counter > 0`: when `self.halving`, do\n   `lfu_counter >>= 1`; else do `lfu_counter -= 1`; return `false`. Otherwise\n   return `true` (victim).\n3. **`steps(i)`.** When `self.halving`, keep\n   `recent + bit_length(counter)`. Otherwise return\n   `recent[i] as usize + lfu_counter`. In both modes each sweep visit still\n   lowers a slot's `steps` by exactly one, so `scan_min_from` (first minimum\n   from the hand, wrapping, early exit at 0) still predicts the sweep victim\n   exactly. Leave `scan_min_from`, `eviction_candidate_index`, `put`'s control\n   flow and `remove_at` as they are.\n4. **Tests (`clock_tests`).**\n   - `stale_high_count_loses_to_recent_low_count` builds `bucket(2)`, which is\n     now linear mode. Set `b.halving = true;` right after construction so it\n     still tests the halving rule.\n   - Add `small_main_uses_linear_decay`: assert `bucket(128).halving == false`\n     and `bucket(1024).halving == true`. In a linear bucket, a counter-5\n     non-recent slot has `steps == 5`, and one sweep visit leaves its counter at 4.\n   - Extend `candidate_matches_sweep` with one forced-halving and one linear\n     run at size 128 (for example seeds 7 and 99, key space 300), so the\n     predictor is checked in both modes.\n   - All other existing tests must pass unchanged. Work through\n     `gclock_frequent_item_outlives_once_hit` under linear steps: key 1 has\n     steps 1+3 = 4 and key 2 has 1+1 = 2, so the victim is still key 2.\n\nInvariants: `items.len() == recent.len() <= max_size`; `lfu_counter <= 7`;\nboth hands in bounds; at most one eviction per miss. The worst-case sweep in\nsmall buckets goes back to 8 rotations of a 64-slot main region (at most 512\nvisits). Large buckets keep the 4-rotation bound.\n\n## Rationale (optional)\n\nCycle 0007 v001's insight blames its only regression (hm_0 @ 16384, from\n0.4529 to 0.4525) on bit-length ranking. It merges counters 2–3 and 4–7 into\none tier each, which costs ranking resolution in the 64-slot main region. Its\nweb_0 gain was at 131072, and v001 found halving adds nothing at 16384 because\nthe short sweep already decays counters quickly. So the two rules can be split\nby main size with no expected cost. The recent bit stays in both modes, so\nsmall buckets now get 9 tiers (recent + 0..7), finer than either parent. This\nanswers the cycle 0007 open question about restoring small-bucket tiers under\nthe recency rule.\n\nRefutation: if hm_0 @ 16384 does not exceed 0.4525, tier resolution was not the\ncause of v001's small-bucket loss. If web_0 @ 16384 drops below 0.4182, linear\ndecay plus the recent bit keeps stale web_0 blocks too long in small main. The\n131072 cells should be bit-identical to v001. Any change there means the mode\ngate is wrong.\n"
}
```

### 15:21:34 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/hypotheses/H02.md",
  "content": "# H02\n\nAdding a one-shot per-slot \"veteran\" bit, set by any hit in main, makes the main sweep evict fresh never-re-hit promotions before decayed counter-0 blocks that have proven reuse in main, raising hit-rate on hm_0 at 131072.\n\n## Parent\n\ncycle/0007/v001\n\n## Changes\n\nEdit only `src/associative_cache/associative_cache_bucket.rs`, starting from\ncycle/0007/v001. Keep `GCLOCK_MAX = 7`, `probation_len`, the probation test,\nthe recent bit, recency-gated halving, and **reset-to-0 on promotion: a\npromoted item enters main with counter 0, recent = false and veteran =\nfalse**. Do not edit `Item` or `src/shared/`.\n\n1. **State.** Add `veteran: Vec<bool>`, created with\n   `Vec::with_capacity(max_size)` and kept index-aligned with `items` and\n   `recent` everywhere they are touched:\n   - fill-up `push(false)`;\n   - every `std::mem::replace` of a slot sets `veteran[slot] = false`. That\n     covers the new item in probation slot `p`, the promoted item in main slot\n     `m`, and the victim slot in plain GCLOCK;\n   - `remove_at` does `veteran.remove(index)`.\n2. **Setting the bit.** In `get_from_position` and in `put`'s existing-key\n   branch, after the counter bump and recent-bit set, also set\n   `veteran[position] = true` **only if the slot is in main**, meaning\n   `!self.split_active() || position >= self.small_len`. Probation hits must\n   not set it, because that would be the non-zero-promotion-credit idea that\n   cycle 0005 retired. Plain-GCLOCK buckets treat every slot as main.\n3. **Sweep rule (`sweep_visit`).** At slot `h`: if `recent[h]`, clear it; else\n   if `lfu_counter > 0`, halve it; else if `veteran[h]`, clear `veteran[h]`\n   (the slot survives exactly one more visit); else `h` is the victim. A later\n   main hit sets the bit again.\n4. **Prediction (`steps`).** Return\n   `recent as usize + bit_length(counter) + veteran as usize`. Each visit\n   still lowers `steps` by exactly one, so `scan_min_from` stays an exact\n   predictor with no other change.\n5. **Tests (`clock_tests`).**\n   - Assert `b.veteran.len() == b.size()` in `check_candidate_matches_sweep`.\n   - `promotion_clears_recent_bit`: also assert `!b.veteran[pos9]`, and that a\n     probation hit (on key 9 before promotion) did not set `veteran`.\n   - Add `fresh_promotion_evicted_before_veteran`: in a size-8 bucket, put a\n     main resident V through hit → full decay to counter 0 with veteran still\n     set, then promote a fresh item F into main. Show that with both at counter\n     0 and no recent bit, the sweep evicts F before V, even when V is reached\n     first from the hand, and that the prediction matches.\n   - Recheck the existing hand-traced tests (`clock_second_chance`,\n     `gclock_frequent_item_outlives_once_hit`,\n     `stale_high_count_loses_to_recent_low_count`). Under the new steps they\n     should still pick the same victims: the hit slots all gain +1 equally.\n     Fix any assertion that doesn't hold and note why.\n\nInvariants: the three per-slot vectors have equal length `<= max_size`; the\nworst-case sweep is 5 rotations; at most one eviction per miss. Metadata grows\nby one byte per slot.\n\n## Rationale (optional)\n\nCycle 0007 found that under reset-to-0, main's victims are almost always\ncounter-0 slots, and it asked whether fresh promotions and decayed veterans\nshould be treated differently among them. Today the hand evicts whichever it\nreaches first. A fresh promotion has only shown one probation-window reuse. A\nveteran has been re-referenced at main-scale distance, which is the\nlong-distance reuse hm_0 depends on and that cycle 0005's ghost recovered\n(at the cost of web_0). This keeps that history without changing admission.\nThe extra grace applies only to blocks hit after entering main, so web_0's\nsingle-return promotions get no extra mass. The bit is consumed once, so it\ncannot trap stale blocks for long.\n\nRefutation: if hm_0 @ 131072 does not beat 0.7383, near-miss history inside\nmain is not the gap. If web_0 @ 131072 falls below 0.5731 by more than hm_0\ngains, veterans are mostly stale once-hot web_0 blocks, which goes against\nv001's finding.\n"
}
```

### 15:21:50 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/hypotheses/H03.md",
  "content": "# H03\n\nIgnoring probation hits that arrive within a short correlated-reference window after insertion (the newest `small_len / 16` probation slots) when deciding promotion raises hit-rate on web_0 at both sizes, by keeping burst-only blocks out of main.\n\n## Parent\n\ncycle/0007/v001\n\n## Changes\n\nEdit only `src/associative_cache/associative_cache_bucket.rs`, starting from\ncycle/0007/v001. The main region, sweep, `steps`, `scan_min_from`, recency-gated\nhalving, `probation_len` and **reset-to-0 on promotion** stay exactly as they\nare. Only the hit path changes, and only for slots in probation.\n\n1. **Constant and helper.** Add `const CRP_DIV: usize = 16;` and a method\n   `fn probation_age(&self, pos: usize) -> usize` that returns\n   `(self.small_hand + self.small_len - 1 - pos) % self.small_len`. It is valid\n   only when `split_active()` and `pos < small_len`. The newest probation\n   insertion sits at `small_hand - 1` (age 0), and the slot at `small_hand` is\n   the oldest (age `small_len - 1`).\n2. **Correlated-hit filter.** Add\n   `fn is_correlated_probation_hit(&self, pos: usize) -> bool`, which returns\n   `self.split_active() && pos < self.small_len && self.probation_age(pos) < self.small_len / CRP_DIV`.\n   The window is 4 slots in 128-slot buckets (`small_len = 64`), 16 slots in\n   1024-slot buckets (`small_len = 256`), and 0 for `small_len < 16`, so tiny\n   buckets and the existing small-size unit tests behave as before.\n3. **Hit paths.** In `get_from_position` and in `put`'s existing-key branch,\n   if `is_correlated_probation_hit(position)`, still set\n   `recent[position] = true` (harmless: probation decisions ignore it and\n   promotion clears it) and still update the value in `put`, but **do not**\n   bump `lfu_counter`. All other hits (later probation hits, main hits,\n   non-split buckets) bump as now. A block re-referenced only inside the window\n   therefore reaches the FIFO head with counter 0 and is evicted. A block that\n   is hit again after the window is promoted as before.\n4. **No other changes.** `eviction_candidate_index` reads only counters, so it\n   stays exact. `remove_at` can shift slots below `small_hand`, which perturbs\n   ages slightly after `evict_item`. That is acceptable, since ages only\n   affect the filter and never bounds.\n5. **Tests (`clock_tests`).**\n   - Existing tests use buckets of size ≤ 8 (window 0) or random workloads, so\n     they must pass unchanged. `candidate_matches_sweep` already covers size\n     128, where the filter is active.\n   - Add `correlated_probation_hit_not_promoted`: in `bucket(128)`, fill it,\n     insert key X by a miss, `get(X)` immediately (age 0), then run 64 more\n     distinct misses. Assert that X was evicted from probation and never\n     reached main.\n   - Add `late_probation_hit_promoted`: same setup, but hit X after 10\n     intervening misses (age 10 ≥ 4). Assert that X ends up in main\n     (`pos >= small_len`) with counter 0.\n\nInvariants: unchanged from v001. No new per-slot state, and the hit path only\ndoes an O(1) modular computation.\n\n## Rationale (optional)\n\nEvery working lever since cycle 0004 has *tightened* admission into main\n(probation itself, the depth floor, reset-to-0), and every loosening failed.\nPromotion is the only admission signal left untouched: any single probation\nhit, however soon after insertion, earns a main slot. This is the classic\nLRU-K / 2Q correlated-reference problem. Back-to-back accesses (such as\nread-after-write or request bursts on web_0) look like reuse but are not\nlong-term reuse. Under reset-to-0 those promotions are exactly the never-re-hit\ncounter-0 residents that cycle 0007 found dominate main's victims. Each one\nalso displaces a real main resident. Requiring the reuse to be separated by\nat least `small_len / 16` insertions filters them while keeping hm_0's\nlong-distance reuse, which by definition arrives later.\n\nRefutation: if web_0 does not rise above 0.4182 @ 16384 or 0.5731 @ 131072,\nearly probation hits are not a meaningful share of wasted promotions. If hm_0\ndrops at either size by more than web_0 gains, short-gap reuse is real reuse\non hm_0, and the window should be dropped, not tuned.\n"
}
```

### 15:21:58 thinking



### 15:21:59 chat

I've written the three cycle 0008 hypotheses. All three build on the current best variant (cycle 0007 v001, hit-rate 0.5455). Each makes one change to `src/associative_cache/associative_cache_bucket.rs` and keeps reset-to-0 on promotion. Nothing has been built or run yet; these are proposals for the implementer.

- **H01: decay rule depends on main-region size.** Small buckets (128 slots, so a 64-slot main region) go back to subtracting one from the counter on each sweep. Their ranking becomes "recent bit + raw counter", which gives 9 tiers. Large buckets (1024 slots) keep v001's halving. It targets hm_0 at 16384, where v001 lost a little (0.4529 → 0.4525) because its coarser ranking merged counter levels. The 131072 results should come out identical to v001.
- **H02: a one-shot "veteran" bit for blocks hit while in main.** When the sweep is choosing among blocks with counter 0, this makes it evict fresh promotions that were never hit again before older blocks that have been reused in main. Probation hits don't set the bit, so it doesn't bring back the extra-credit-on-promotion idea retired in cycle 0005. It targets hm_0 at 131072. Last cycle's insight raised this choice among counter-0 blocks as an open question.
- **H03: ignore probation hits that come immediately after insertion.** A hit doesn't count toward promotion if it lands within the first `small_len / 16` insertions: 4 slots in small buckets, 16 in large ones. This keeps bursts of back-to-back accesses out of main. It targets web_0 at both sizes. Promotion is the one admission signal not yet tightened, and tightening admission is the only kind of change that has worked so far. The window is zero in buckets of 8 slots or fewer, so the existing small-bucket unit tests behave as before.

Each file names the tests to add or adjust and gives explicit numbers that would refute the claim. One change to existing tests: H01 makes the 2-slot test bucket use the subtract-one rule, so `stale_high_count_loses_to_recent_low_count` has to force halving on to keep testing the old rule.

Files are in `cycles/0008/hypotheses/`:
- H01.md
- H02.md
- H03.md


---

- finished: 2026-10-28T15:21:59+00:00
- duration: 126.79s
- status: ok
- input_tokens: 22
- output_tokens: 12618
- cache_read_tokens: 359869
- cache_write_tokens: 43985
- cost_usd: 0.676302
