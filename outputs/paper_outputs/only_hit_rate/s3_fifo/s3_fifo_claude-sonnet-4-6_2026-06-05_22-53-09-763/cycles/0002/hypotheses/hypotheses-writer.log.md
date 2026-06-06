# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0002/hypotheses
- allow_edits: True
- started: 2026-06-05T23:04:11+00:00

## timeline

### 23:04:14 thinking

Let me read all the context files to understand the current state and craft well-grounded hypotheses.

### 23:04:21 tool: Agent

```json
{
  "description": "Read all context files for cycle 0002 hypotheses",
  "subagent_type": "Explore",
  "prompt": "Read the following files and return their full contents. I need all content verbatim to write research hypotheses.\n\n1. /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/cycle-winners.json\n2. /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/cycle-insight.md\n3. /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/leaderboard.md\n4. /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles-ledger.jsonl\n5. /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cross-cycle-insights.md  (if it exists)\n6. List all variant directories under /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/\n\nThen for each variant directory found (v001, v002, v003, etc.), read:\n- The fitness-vector.json: /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v<NNN>/eval/fitness-vector.json\n- The insight.md: /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v<NNN>/insight.md\n- The overlay files in the latest attempt directory under /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v<NNN>/attempts/ — list what's there and read each source file.\n\nAlso read the baseline source files:\n- List files in /home/user/cache-evolve/inputs/subjects/s3_fifo/baseline/\n- Read each .rs file found there\n\nReturn all contents verbatim."
}
```

### 23:04:22 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/cycle-winners.json"
}
```

### 23:04:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/cycle-insight.md"
}
```

### 23:04:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/leaderboard.md"
}
```

### 23:04:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles-ledger.jsonl"
}
```

### 23:04:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cross-cycle-insights.md"
}
```

### 23:04:24 tool: Bash

```json
{
  "command": "ls -1 /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/ | sort"
}
```

### 23:04:24 tool: Bash

```json
{
  "command": "ls -1 /home/user/cache-evolve/inputs/subjects/s3_fifo/baseline/ | sort"
}
```

### 23:04:27 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v*",
  "path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants"
}
```

### 23:04:27 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/s3_fifo/baseline/*.rs",
  "path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/baseline"
}
```

### 23:04:29 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/*/"
}
```

### 23:04:29 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/s3_fifo/baseline/"
}
```

### 23:04:31 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001"
}
```

### 23:04:31 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/baseline"
}
```

### 23:04:33 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/**/*"
}
```

### 23:04:33 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/s3_fifo/baseline/**/*"
}
```

### 23:04:34 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/s3_fifo/**/*.rs"
}
```

### 23:04:37 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v001/eval/fitness-vector.json"
}
```

### 23:04:37 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v001/insight.md"
}
```

### 23:04:37 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v002/eval/fitness-vector.json"
}
```

### 23:04:37 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v002/insight.md"
}
```

### 23:04:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v003/eval/fitness-vector.json"
}
```

### 23:04:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v003/insight.md"
}
```

### 23:04:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v001/task.md"
}
```

### 23:04:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v001/attempts/01/task.md"
}
```

### 23:04:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v001/attempts/01/overlay/src/s3_fifo/mod.rs"
}
```

### 23:04:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v002/task.md"
}
```

### 23:04:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v002/attempts/01/task.md"
}
```

### 23:04:43 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v002/attempts/01/overlay/src/s3_fifo/mod.rs"
}
```

### 23:04:48 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v003/task.md"
}
```

### 23:04:48 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v003/attempts/01/task.md"
}
```

### 23:04:49 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/variants/v003/attempts/01/overlay/src/s3_fifo/mod.rs"
}
```

### 23:04:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/src/s3_fifo/mod.rs"
}
```

### 23:04:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/src/lib.rs"
}
```

### 23:04:52 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/hypotheses/**/*"
}
```

### 23:04:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/hypotheses/H01.md"
}
```

### 23:04:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/hypotheses/H02.md"
}
```

### 23:04:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0001/hypotheses/H03.md"
}
```

### 23:04:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/s3_fifo/baseline/insight.md"
}
```

### 23:07:47 thinking

Now I have all the context I need to craft three distinct hypotheses for cycle 0002. Let me synthesize what I know:

**What we know:**
1. **v001 (winner, 0.7248)**: Shrinks small queue from 25% to 10% (`capacity / 10`). This is the incumbent best.
2. **v002 (loser, 0.7136)**: Lowered promotion threshold from `freq > 1` to `freq > 0`. Hurt hit rate - too permissive, flooded main with borderline entries.


3. **v003 (loser, 0.7091)**: Doubled ghost capacity to `main_size * 2`. Hurt hit rate - larger ghost added noise, not signal.

**Cross-cycle lessons:**
- Shrinking small queue to give main more resident capacity = WIN
- Relaxing admission criteria = LOSE
- Bigger ghost window = LOSE
- The second-chance mechanism in main is a lagging corrective, not a robust filter

**Open questions from cycle-insight.md:**
- Is 10% the floor, or does 5-8% keep improving?
- Does the optimal ghost queue size lie *below* the current baseline cap?
- Does v001's 10% represent the sweet spot, or is there room to push further?

**Constraints:**
- At least one hypothesis MUST cite `cycle/0001/v001` as parent
- The other two can cite baseline or prior losers
- All edits must be in `src/s3_fifo/mod.rs` (not the gate-integrity surface)

**Three distinct strategies for cycle 0002:**

**H01 (build on v001 - required)**: Fine-tune small queue sizing around v001's 10% allocation. I want to test whether going smaller (8% or 5%) improves hit rate further, or if going slightly larger (12%) finds a better equilibrium. The cycle insight suggests there's a sweet spot somewhere in this range that balances main queue capacity with small queue filtering.

**H02 (different direction, baseline parent)**: Shrink the ghost queue from `main_size` to `main_size / 2` to reduce noise from distant evictions and improve recency precision. v003 expanded it and failed, so contracting it might be the right move.

**H03 (another direction)**: Combine v001's small queue approach with a reduced ghost queue size, testing whether the two improvements work synergistically. This also cites v001 as parent since I can have multiple hypotheses building on it.

Actually, let me reconsider what makes these truly distinct strategies—I need to ensure H03 is genuinely different from just stacking two parameters.

The core untested hypotheses I should focus on are: pushing the small queue ratio even lower than v001's 10% to see if there's still room to optimize, testing whether a smaller ghost queue improves precision without sacrificing coverage, and exploring whether adjusting MAX_FREQ could change how items cycle through the cache and affect eviction costs. Reducing the small queue from 10% to 5% of capacity frees up more slots for the main queue's second-chance policy, which should boost hit rate by giving hot items more room to stay resident. For the ghost queue, I'm testing whether halving its size to match half the main queue improves precision by focusing on only the most recent evictions rather than tracking distant re-accesses that don't help this workload. Defining ghost queue capacity... only one "second chance" reinsertion before becoming eviction candidates, reducing the average scan depth in `evict_main()`. But I'm second-guessing whether lowering MAX_FREQ from 3 to 2 actually helps—it makes second-chance more decisive and reduces scanning, but it might also evict moderately hot items too aggressively. With MAX_FREQ=3, items get three chances to stay; with MAX_FREQ=2, they get two, cycling out faster. For a workload with strong temporal locality like workload-a, keeping items longer might be better than the scan savings.

But v001 proved that capacity improvements beat admission policy tweaks, and the real bottleneck is second-chance eviction when flooded with low-quality entries. Since v001 uses strict frequency thresholds for promotion, second-chance can work more efficiently there, so lowering MAX_FREQ to 2 might actually be the right move to keep it lean while still protecting genuinely hot items.

I'm considering an alternative approach: instead of just lowering MAX_FREQ, I could combine v001's 10% small queue with a smaller ghost queue set to `main_size / 2`. This tests whether the two changes work synergistically—more main capacity plus better ghost precision. But mixing two changes makes it harder to isolate what's actually working.

So I'm settling on three separate hypotheses: H01 tests if 5% small queue beats v001's 10%, H02 tests ghost shrinking alone from baseline, and H03 tests ghost shrinking on top of v001. This way I can see if ghost precision needs the larger main queue to be effective, or if they're independent improvements.

Actually, H02 and H03 are too similar mechanically—both just shrinking the ghost queue. Let me make H03 distinct by tuning the MAX_FREQ parameter instead, lowering it from 3 to 2 on top of v001 to test whether the second-chance mechanism becomes more decisive with an enlarged main queue.

So my final three: H01 shrinks small to 8% on v001, H02 shrinks ghost to half the main size on baseline, and H03 keeps v001's 10% small but caps MAX_FREQ at 2 to test if tighter second-chance eviction helps when the main queue is bigger.

Now I'm checking whether capacity/12 is the right intermediate value for H01. Looking at the math for a 2048-capacity cache, capacity/12 gives about 170 slots for small (8.3%), which sits between v001's 10% and the lower 5% bound mentioned in the cycle insight. I could also test capacity/20 for a more aggressive 5% shrink, but capacity/12 seems like a reasonable middle ground to explore whether fine-tuning around 10% yields further improvements.

I'm settling on three concrete experiments: H01 tests the moderate 8.3% small-queue reduction, H02 shrinks the ghost queue to improve precision, and H03 lowers MAX_FREQ to make the second-chance mechanism more aggressive. These should give me a clearer picture of whether the enlarged main queue benefits from tighter hot-set cycling. I need to be careful about the promotion threshold though—the cross-cycle insights warn against relaxing it without a compensating mechanism to protect against thrashing.

For H03 specifically, the logic is that with a larger main queue from H01's sizing, I can afford to evict items faster by dropping MAX_FREQ from 3 to 2, reducing wasted CPU on re-queuing marginally-hot items while still preserving genuinely re-accessed entries. Let me verify the evict_main loop behavior...

Checking the eviction logic...

The second-chance mechanism means items with freq=3 get three re-insertions before eviction, but with MAX_FREQ=2 they only get two, so they exit the loop faster on average. For hit rate, this matters because items accessed exactly 3 times would be evicted sooner under the lower threshold—whether that helps or hurts depends on whether the workload has many such items that should stay resident.

I'm skeptical that MAX_FREQ alone would improve hit rate meaningfully. Let me explore other approaches: resetting freq to 0 when promoting ghost entries seems risky and doesn't match the current initialization; combining a ghost queue with a half-size cap on top of v001 could test whether the two ideas synergize; or I could look at what happens when...

Actually, the most promising angle is testing whether a smaller ghost queue works better when paired with v001's larger main queue—the intuition being that with more capacity in main, fewer items reach eviction, so a smaller ghost window might still catch most re-accesses while reducing overhead.

So I'm settling on three hypotheses: H01 tests even finer capacity partitioning at 8.3%, H02 isolates ghost sizing at half the main size, and H03 combines v001's improvements with the smaller ghost to see if they work synergistically. Now I need to write out the actual hypothesis configuration files.

Writing hypothesis documentation... Updating the ghost queue overflow check in `evict_small()` to use the new `ghost_size` field instead of `main_size`. The small queue capacity and promotion threshold remain unchanged from the baseline.

For H03, I'm combining the enlarged main queue from v001 with the half-size ghost queue approach, computing `ghost_size` as `main_size / 2` and updating the same overflow guard in `evict_small()`.

Now I'm writing out the implementation files.

### 23:07:52 chat

Now I have all the context I need. Let me synthesize the findings and write the three hypotheses:

**Key insights from cycle 0001:**
- v001 (winner, 0.7248): Shrinking small queue 25%→10% gave main more capacity → WIN
- v002 (0.7136): Lowered promotion threshold (freq > 1 → freq > 0) flooded main → LOSE  
- v003 (0.7091): Doubled ghost queue → added noise, not signal → LOSE
- Open questions: Is 10% the floor? Does a *smaller* ghost beat the baseline cap? Does 10% change the optimal threshold?

**Three distinct strategies:**
1. **H01** (parent: v001) — push small queue even lower to `capacity/12` (~8.3%) to test if 10% is still too generous
2. **H02** (parent: baseline) — shrink ghost to `main_size/2` to test precision-over-coverage alone, since v003 showed bigger ghost = worse
3. **H03** (parent: v001) — layer the half-size ghost on top of v001's enlarged main, testing if both wins are synergistic

### 23:07:52 tool: Bash

```json
{
  "command": "mkdir -p /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0002/hypotheses",
  "description": "Create hypotheses output directory"
}
```

### 23:08:01 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0002/hypotheses/H01.md",
  "content": "# H01\n\nReducing the small queue from 10% to ~8% of capacity (`capacity / 12`) further enlarges the main queue's resident footprint and raises hit rate beyond cycle/0001/v001.\n\n## Parent\n\ncycle/0001/v001\n\n## Changes\n\nIn `try_new()` in `src/s3_fifo/mod.rs`, change the `small_size` calculation from\n\n```rust\n(capacity / 10).max(1).min(capacity - 1)\n```\n\nto\n\n```rust\n(capacity / 12).max(1).min(capacity - 1)\n```\n\nFor a 2048-slot cache this gives `small_size = 170` (vs 204 in v001), adding 34 more resident slots to main (1878 vs 1844). The `main_size` calculation (`capacity - small_size`) is unchanged, so the ghost queue capacity grows correspondingly. All other logic — the promotion threshold (`freq > 1`), `MAX_FREQ`, `evict_main`, `evict_small`, and the ghost-hit re-admission path — remains identical to v001.\n\n## Rationale\n\nThe cycle-0001 insight explicitly left open whether 10% is the floor or whether 8% continues the upward trajectory. This hypothesis takes one calibrated step downward from v001's winning ratio, keeping the small queue large enough to absorb cold bursts (170 slots at capacity=2048) while reclaiming 34 more slots for the main queue's second-chance retention pool. If the hit-rate improvement from 25%→10% was proportional to the main-queue capacity gained, a further 34-slot gain should yield a proportional — if smaller — improvement.\n"
}
```

### 23:08:12 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0002/hypotheses/H02.md",
  "content": "# H02\n\nCapping the ghost queue at half of `main_size` (`main_size / 2`) sharpens re-access signal precision and raises hit rate by eliminating noise from distant, non-recurring evictions.\n\n## Parent\n\nbaseline\n\n## Changes\n\nMake three targeted edits to `src/s3_fifo/mod.rs`:\n\n1. Add a `ghost_size` field to the `S3Fifo` struct:\n\n```rust\npub struct S3Fifo<K: PartialEq + Clone, V> {\n    small: VecDeque<Entry<K, V>>,\n    main: VecDeque<Entry<K, V>>,\n    ghost: VecDeque<K>,\n    small_size: usize,\n    main_size: usize,\n    ghost_size: usize,   // new field\n}\n```\n\n2. In `try_new()`, compute `ghost_size` as half of `main_size` and wire it into construction:\n\n```rust\nlet ghost_size = (main_size / 2).max(1);\nSome(Self {\n    small: VecDeque::with_capacity(small_size),\n    main: VecDeque::with_capacity(main_size),\n    ghost: VecDeque::with_capacity(ghost_size),\n    small_size,\n    main_size,\n    ghost_size,\n})\n```\n\n3. In `evict_small()`, replace the overflow guard from `self.main_size` to `self.ghost_size`:\n\n```rust\nif self.ghost.len() >= self.ghost_size {\n    self.ghost.pop_back();\n}\n```\n\nThe small queue stays at baseline (`capacity / 4`). The promotion threshold (`freq > 1`) and all remaining logic are unchanged. For a 2048-slot cache this shrinks the ghost window from 1536 keys to 768 keys.\n\n## Rationale\n\nCycle 0001's v003 showed that doubling the ghost window (1536→3072) degraded hit rate because the extended history captured distant re-accesses — single appearances of long-ago evicted keys — that are noise for workload-a's LIRS trace. The cross-cycle insight stated that \"precision of ghost tracking matters more than coverage.\" This hypothesis tests the opposite extreme from v003: halving the baseline ghost window to 768 keys keeps only very recent evictions in view, so every ghost hit corresponds to a key evicted within the last ~768 insertions. If the workload's re-use distance is shorter than the baseline window, removing the tail of the ghost queue strips out the noise without losing any true re-access signal.\n"
}
```

### 23:08:24 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0002/hypotheses/H03.md",
  "content": "# H03\n\nCombining v001's enlarged main queue (small = 10%) with a half-size ghost queue raises hit rate beyond v001 alone by layering sharper ghost signal precision on top of the increased resident capacity.\n\n## Parent\n\ncycle/0001/v001\n\n## Changes\n\nStarting from v001's overlay (which already sets `small_size = capacity / 10`), make three additions to `src/s3_fifo/mod.rs`:\n\n1. Add a `ghost_size` field to the `S3Fifo` struct:\n\n```rust\npub struct S3Fifo<K: PartialEq + Clone, V> {\n    small: VecDeque<Entry<K, V>>,\n    main: VecDeque<Entry<K, V>>,\n    ghost: VecDeque<K>,\n    small_size: usize,\n    main_size: usize,\n    ghost_size: usize,   // new field\n}\n```\n\n2. In `try_new()`, after computing `main_size` (which in v001 is `capacity - capacity/10`), compute `ghost_size` and wire it in:\n\n```rust\nlet ghost_size = (main_size / 2).max(1);\nSome(Self {\n    small: VecDeque::with_capacity(small_size),\n    main: VecDeque::with_capacity(main_size),\n    ghost: VecDeque::with_capacity(ghost_size),\n    small_size,\n    main_size,\n    ghost_size,\n})\n```\n\n3. In `evict_small()`, update the ghost overflow guard from `self.main_size` to `self.ghost_size`:\n\n```rust\nif self.ghost.len() >= self.ghost_size {\n    self.ghost.pop_back();\n}\n```\n\nThe 10% small queue sizing from v001 (`capacity / 10`) is preserved. Only the ghost capacity changes: for a 2048-slot cache, the ghost shrinks from 1844 (v001's `main_size`) to ~922 keys. The promotion threshold (`freq > 1`), `MAX_FREQ`, and all eviction logic remain as in v001.\n\n## Rationale\n\nv001 grew main to 1844 slots. With more capacity in main, items cycle through main more slowly on average, meaning each eviction from main is of a slightly colder item than in the baseline. A full-main_size ghost window (1844 keys) may therefore capture more noise than before: items evicted from the now-larger main include items that were genuinely cold by the time they left, and their ghost entries inflate the tracking history with keys unlikely to benefit from direct promotion. Halving the ghost to 922 keys keeps only the most recently evicted items tracked — precisely where re-use signal is densest — while shedding the long tail of the window that v003's failure showed carries noise. The two changes are expected to be complementary: v001 maximises the value of a cache slot; the ghost shrinkage maximises the value of a ghost entry.\n"
}
```

### 23:08:35 chat

All three hypothesis files are written. Here's the cohort summary:

---

**Cycle 0002 — 3 Hypotheses**

| File | Claim | Parent | Mechanism |
|------|-------|--------|-----------|
| **H01** | Reduce small queue from 10% → ~8% (`capacity/12`) raises hit rate beyond v001 | `cycle/0001/v001` | Push small-queue floor lower; +34 resident slots in main for 2048-slot cache |
| **H02** | Cap ghost queue at `main_size/2` (768 keys) raises hit rate by eliminating noise from distant evictions | `baseline` | Ghost precision: only the most recent ~768 evictions tracked, cutting v003-style noise |
| **H03** | Combine v001's 10% small queue with a half-size ghost raises hit rate further by stacking capacity and precision gains | `cycle/0001/v001` | Synergy test: v001's enlarged main + sharper ghost window together |

**Rationale for the cohort design:**
- H01 directly answers the cycle-insight open question "is 10% the floor?" with a measured step to ~8.3%  
- H02 answers the second open question "does a smaller-than-baseline ghost improve precision?" in isolation from the baseline, giving a clean comparison point  
- H03 tests the same ghost-shrink idea on top of v001, asking whether the two improvements compound — and providing a diagnostic: if H03 > H02 > baseline but H03 ≈ H01, the ghost shrink adds nothing on top of capacity gains; if H03 > H01 > v001, they stack


---

- finished: 2026-06-05T23:08:35+00:00
- duration: 263.55s
- status: ok
- input_tokens: 7
- output_tokens: 8505
- cache_read_tokens: 155581
- cache_write_tokens: 23156
- cost_usd: 0.398615
