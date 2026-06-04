# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/hypotheses
- allow_edits: True
- started: 2026-06-04T04:47:23+00:00

## timeline

### 04:47:26 thinking

Let me read all the context files to understand the current state of the research and formulate good hypotheses for cycle 0003.

### 04:47:27 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/variants/*/eval/fitness-vector.json"
}
```

### 04:47:27 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/cycle-winners.json"
}
```

### 04:47:28 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/cycle-insight.md"
}
```

### 04:47:30 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/leaderboard.md"
}
```

### 04:47:31 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cross-cycle-insights.md"
}
```

### 04:47:33 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/variants/*/attempts/*/overlay/**"
}
```

### 04:47:35 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/variants/**"
}
```

### 04:47:38 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/variants/v001/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 04:47:38 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/variants/v001/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 04:47:41 thinking

Now I have a good understanding of the current state. Let me also read the baseline and cycle 0001 variants for more context.

### 04:47:43 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/variants/v001/insight.md"
}
```

### 04:47:44 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/variants/v002/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 04:47:47 thinking

Now let me read the baseline to understand the original implementation, and also the v003 variant from cycle 0002 to understand what was tried.

### 04:47:48 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/inputs/subjects/associative_cache/baseline/**"
}
```

### 04:47:49 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0002/variants/v003/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 04:49:11 thinking

Now I have all the context needed to generate the hypotheses. Let me summarize what I know:

1. **Baseline**: Pure ClassicLRU with hit rate 0.661
2. **Cycle 0001 v001**: LFU-weighted eviction via `ClassicLRU` (sorting by `lfu_counter` instead of recency); hit rate 0.723
3. **Cycle 0001 v003**: Second-chance stash; hit rate 0.689
4. **Cycle 0002 v001 (WINNER)**: LFU-weighted eviction + second-chance stash composition; hit rate 0.741


5. **Cycle 0002 v002 & v003**: Counter capping and frequency-recency product scoring both showed no improvement over the baseline

The winning approach combines LFU-weighted eviction that prioritizes items by access frequency with a per-bucket second-chance stash that gives recently evicted items a chance to be restored before permanent removal.

Now I'm thinking through what still needs investigation: whether the fixed stash size of `bucket_size / 4` is optimal, or if it should adapt based on eviction patterns; and whether stash-promoted items should receive a stronger counter boost than newly inserted items to better compete in the primary bucket.

I'm also considering whether the stash itself should use LFU eviction instead of FIFO, and whether items sitting in the stash should have their counters reset or decay over time so they don't artificially block fresher candidates. Finally, there's the question of preventing items from being promoted back to primary too aggressively after a long period of inactivity in the stash.

Let me focus on three concrete hypotheses to test: increasing the stash size from a quarter to half the bucket size to capture more medium-frequency items, switching to LFU-based eviction when the stash fills up instead of dropping the oldest item, and...

Actually, I'm realizing H03 might backfire — if I reset the counter to 0 on stash entry, promoted items start competing at a disadvantage against genuinely hot primary items that have accumulated higher counters.

The real problem is that stash items with very high counters are likely "stale hot" — they were accessed frequently in the past but haven't been touched recently, which is why they got evicted in the first place. When they return to primary, their inflated counter makes them rank too high in LFU terms and blocks actually hot items. So I should cap or reduce the counter at promotion time instead, maybe setting it to something like half the original value plus one, or just resetting it to 1 to let it compete fairly.

Looking at the promotion logic...

Examining promotion logic... if an item had lfu_counter = 10 when evicted to stash, after promotion it has lfu_counter = 11. This item now looks extremely hot. But actually... it WAS accessed again (that's why it's being promoted), so maybe keeping a high counter is reasonable.

Hmm. The concern is more about items in the stash that are NOT accessed — they just stay in the stash with a high counter until they're dropped. But those items don't affect primary eviction since they're in the stash.

Let me reconsider the three hypotheses:

**H01: Larger stash (bucket_size / 2 instead of bucket_size / 4)** — targets hit-rate by capturing more medium-frequency items. I'd increase the stash capacity from a quarter of the bucket size to half, which should help retain more candidates during cold bursts.

**H02: LFU-ordered stash eviction (drop lowest-frequency from stash when stash is full)** — targets hit-rate by retaining more productive stash candidates. Instead of dropping the oldest stash item when full, I'd scan for and evict the one with the minimum frequency counter, since the stash is small enough that this scan is cheap.

**H03: Lazy decay on stash entry** — when an item is evicted to stash, I'd reset or halve its frequency counter to prevent stale-hot items from dominating the primary cache after promotion. But actually, if an item is being promoted from stash, it means it was accessed again, so the high counter might be justified.

The real issue is more subtle: an item with a high counter from the past gets evicted, then promoted back with that same high counter intact, and now it's hard to evict again even if it's no longer frequently used. This could cause a stale item to camp in primary indefinitely just because it was hot once.

I could decay the counter when moving to stash—halving it, say—to mitigate this. That's actually similar in spirit to the counter cap approach in an earlier variant, except applied at stash entry instead of during hits.

Now I'm exploring a two-tier stash idea: separate cold items (never re-accessed in stash) from warm ones (accessed while stashed), and prioritize evicting from the cold tier first. But this gets complicated quickly.

A simpler angle: ghost entries for dropped stash items. When something overflows from the stash, I'd record its key in a small ghost list. If that key gets inserted again, I'd give it a higher initial counter to reflect its multi-phase access pattern and protect it from quick eviction.

Or even better—dynamically adjust the stash size itself based on promotion rate. If I'm seeing lots of stash promotions, grow the stash; if drops are frequent with few promotions, shrink it. This adapts the stash capacity to actual workload behavior.

But both of these are getting complicated to implement correctly. Let me reconsider whether swapping the search order in `get()` could work—checking stash before primary instead of after. Actually, that doesn't help because stashed items were already evicted from primary, so searching stash first wouldn't catch anything primary would miss. And items can't exist in both places simultaneously due to the promotion invariant, so the order doesn't matter.

Since both approaches are linear scans anyway, the real question is what levers I actually have to tune: the eviction policy for primary, stash size, stash eviction strategy, how much the counter increments on promotion, and a few other parameters.

Looking back at what's been explored, LFU eviction already won out in the first cycle, stash size is still open, and the stash eviction policy hasn't been tested yet. There's also an interesting mechanism I haven't tried: when inserting a brand new item with counter 0, I could scan the stash first to see if any item there has a higher counter, and if so, swap it back into primary instead of evicting something from primary.

But thinking through this more carefully—a new item starts at counter 0, which is already the minimum possible frequency, so it would just become the next eviction candidate anyway. The current approach of evicting the LFU item from primary to the stash is actually fine. Though there's still a potential variation worth considering: when inserting a new key into a full primary, if the stash already contains items with very low frequencies, maybe there's a smarter way to handle the insertion.

Let me lay out three concrete hypotheses to test. The first increases the stash capacity to half the bucket size. The second changes how items are evicted from the stash—always removing the minimum-frequency item instead of using FIFO. The third modifies the insertion logic so that when a new key (counter 0) needs to go into a full primary, we check if any stash item has a higher frequency than what we're about to evict from primary, and if so, we swap them—keeping the higher-frequency item in primary and placing the new item in the stash instead.

Actually, let me think through H03 more carefully. I've settled on three distinct hypotheses that each target a different aspect of the stash mechanism: increasing stash capacity to reduce overflow losses, switching to frequency-ordered eviction instead of FIFO to retain higher-value items, and boosting the promotion credit so items don't get immediately re-evicted. Now I'm ready to write out the output files and set up the directory structure.

### 04:49:11 chat

Now I have enough context to synthesize the three hypotheses. Let me check the output directory:

### 04:49:12 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/hypotheses 2>/dev/null || echo \"directory not found\"",
  "description": "Check if output directory exists"
}
```

### 04:49:27 thinking

Good, the directory exists. Now let me write the three hypothesis files.

My three hypotheses:

1. **H01: Larger stash (bucket_size / 2)** — from cycle/0002/v001
   - Change: In `AssociativeCacheBucket::new()`, change `(settings.bucket_size / 4).max(1)` to `(settings.bucket_size / 2).max(1)`
   - Claim: Doubling the stash capacity from bucket_size/4 to bucket_size/2 will increase hit rate by capturing more medium-frequency items before stash overflow discards them.


2. **H02: LFU-ordered stash eviction** — from cycle/0002/v001
   - Change: In `stash_push()`, replace the FIFO removal with logic that finds and removes the item with the lowest frequency counter, preserving higher-frequency items longer.
   - Claim: Switching from FIFO to frequency-based stash eviction will boost hit rate by keeping frequently-accessed items available for promotion.
3. **H03: Higher lfu_counter credit on stash promotion** — from cycle/0002/v001
   - Change: In `get()` and `put()` methods, increase the frequency counter boost when promoting items from stash to main bucket.
   - Claim: Giving promoted items a higher frequency credit will prevent them from being immediately evicted, improving their chances of staying in the cache longer.

These are all distinct, testable strategies grounded in the same codebase.

### 04:49:34 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/hypotheses/H01.md",
  "content": "# H01\n\nDoubling the per-bucket stash capacity from `bucket_size / 4` to `bucket_size / 2` will increase hit rate by retaining more medium-frequency items through cold-burst insertions before stash overflow discards them.\n\n## Parent\n\ncycle/0002/v001\n\n## Changes\n\nIn `src/associative_cache/associative_cache_bucket.rs`, in `AssociativeCacheBucket::new()`, change the `stash_max_size` initialisation line from:\n\n```rust\nstash_max_size: (settings.bucket_size / 4).max(1),\n```\n\nto:\n\n```rust\nstash_max_size: (settings.bucket_size / 2).max(1),\n```\n\nNo other changes. The `stash_push()`, `evict_to_stash()`, `get()`, and `put()` logic are all preserved exactly as in the parent; only the stash capacity bound changes. The invariant that primary items and stash items are disjoint is unaffected because it is enforced by the promotion-and-remove path, not by the size limit.\n\n## Rationale\n\nThe cycle-insight open question asks whether `bucket_size / 4` is the right stash size. A small stash caps how many displaced items can survive a burst; if the hot set plus the burst together exceed `primary + stash` capacity the oldest stash entries are dropped by FIFO before they can be re-accessed. A larger stash extends the survival window at the cost of slightly more memory per bucket. This hypothesis isolates the capacity dimension from the eviction-within-stash dimension tested by H02.\n"
}
```

### 04:49:46 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/hypotheses/H02.md",
  "content": "# H02\n\nReplacing the FIFO stash eviction policy with a drop-minimum-frequency selection will increase hit rate by retaining higher-frequency stash candidates, so that any given stash promotion is more productive than under the current oldest-first rule.\n\n## Parent\n\ncycle/0002/v001\n\n## Changes\n\nIn `src/associative_cache/associative_cache_bucket.rs`, modify the `stash_push()` method. Currently, when the stash is at capacity, index 0 (the oldest entry) is unconditionally removed:\n\n```rust\nfn stash_push(&mut self, item: Item<K, V>) {\n    if self.stash.len() >= self.stash_max_size {\n        self.stash.remove(0);\n    }\n    self.stash.push(item);\n}\n```\n\nReplace the `self.stash.remove(0)` line with a scan that finds and removes the stash entry whose `lfu_counter` is smallest (ties broken by position — earliest position wins, i.e. preserve youngest among ties):\n\n```rust\nfn stash_push(&mut self, item: Item<K, V>) {\n    if self.stash.len() >= self.stash_max_size {\n        let victim = self\n            .stash\n            .iter()\n            .enumerate()\n            .min_by_key(|(i, s)| (s.lfu_counter, *i))\n            .map(|(i, _)| i)\n            .unwrap_or(0);\n        self.stash.remove(victim);\n    }\n    self.stash.push(item);\n}\n```\n\nAll other methods (`evict_to_stash`, `get`, `put`) remain identical to the parent. The stash size cap and the primary-eviction path are unchanged.\n\n## Rationale\n\nUnder FIFO, an item with `lfu_counter = 0` inserted long ago blocks a higher-frequency item that was evicted later. Selecting the minimum-frequency victim ensures the stash population trends toward items that have actually been re-accessed, mirroring the same LFU principle that drives primary eviction in the parent variant. The scan cost is at most `stash_max_size` comparisons per insertion — at most 4–8 items for typical `bucket_size` values — which is the same order as the existing linear stash search in `get()` and `put()`.\n"
}
```

### 04:49:56 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0003/hypotheses/H03.md",
  "content": "# H03\n\nIncreasing the lfu_counter credit awarded on stash promotion from 1 to 2 will increase hit rate by giving promoted items enough of a head start in primary to survive the next cold-burst insertion cycle rather than being immediately re-evicted.\n\n## Parent\n\ncycle/0002/v001\n\n## Changes\n\nIn `src/associative_cache/associative_cache_bucket.rs`, change every stash-promotion site where `stash_item.lfu_counter += 1` appears to `stash_item.lfu_counter += 2`. There are two such sites:\n\n1. In `put()`, the Case 2 branch (key found in stash):\n   ```rust\n   // before\n   stash_item.lfu_counter += 1;\n   // after\n   stash_item.lfu_counter += 2;\n   ```\n\n2. In `get()`, the stash-hit promotion branch:\n   ```rust\n   // before\n   stash_item.lfu_counter += 1;\n   // after\n   stash_item.lfu_counter += 2;\n   ```\n\nNo other changes. The `stash_push()`, `evict_to_stash()`, and primary eviction logic are preserved exactly.\n\n## Rationale\n\nA stash item that arrives in primary with `lfu_counter = original + 1` competes directly against items with `lfu_counter ≥ 1` that were never evicted. On the next insertion that requires an eviction, the promoted item — if its counter is only 1 — may be selected as the LFU victim again, sending it back to the stash immediately. This round-trip consumes a stash slot without delivering a hit. A credit of +2 gives the promoted item one extra eviction-cycle of protection, increasing the probability it accumulates a further primary hit before its next eviction. The parent already showed that the stash mechanism is productive; this hypothesis tests whether the credit size is the binding constraint on how long promoted items persist in primary.\n"
}
```

### 04:50:04 chat

All three hypothesis files are written. Here is a summary of what was generated:

---

**H01 — Larger stash capacity** (`cycle/0002/v001`)  
Changes `stash_max_size` from `bucket_size / 4` to `bucket_size / 2` in `AssociativeCacheBucket::new()`. Tests whether the current stash is undersized and loses hit-rate due to premature FIFO overflow — the direct answer to the open question from the cycle-2 insight.

**H02 — LFU-ordered stash eviction** (`cycle/0002/v001`)  
Replaces the FIFO drop in `stash_push()` with a minimum-`lfu_counter` scan, exactly mirroring the LFU principle used for primary eviction. Keeps the highest-frequency items in the stash longer, so any given stash promotion is more productive than under oldest-first.

**H03 — Higher stash-promotion credit** (`cycle/0002/v001`)  
Changes both stash-promotion sites (`get()` stash-hit path and `put()` case-2 path) from `lfu_counter += 1` to `lfu_counter += 2`. Tests whether a promoted item with only +1 credit is being immediately re-evicted on the next cold-burst insertion before it can accumulate a further hit — a round-trip that burns a stash slot without a payoff.

All three hypotheses are distinct, grounded in the incumbent winner, and each targets a different parameter of the stash mechanism.


---

- finished: 2026-06-04T04:50:04+00:00
- duration: 161.53s
- status: ok
- input_tokens: 13
- output_tokens: 8189
- cache_read_tokens: 326241
- cache_write_tokens: 32370
- cost_usd: 0.343546
