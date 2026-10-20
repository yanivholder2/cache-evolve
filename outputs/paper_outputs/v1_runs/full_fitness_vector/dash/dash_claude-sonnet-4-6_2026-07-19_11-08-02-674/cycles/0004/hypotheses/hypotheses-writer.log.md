# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0004\hypotheses
- allow_edits: True
- started: 2026-07-19T09:24:32+00:00

## timeline

### 09:24:39 thinking

Let me read all the context files to understand the current state of the evolution and then generate 3 distinct hypotheses for cycle 0004.

### 09:24:40 tool: Bash

```json
{
  "command": "ls \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\"",
  "description": "List cycle 0003 variants"
}
```

### 09:24:40 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\cycle-winners.json"
}
```

### 09:24:41 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\cycle-insight.md"
}
```

### 09:24:41 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\leaderboard.md"
}
```

### 09:24:42 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cross-cycle-insights.md"
}
```

### 09:24:42 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles-ledger.jsonl"
}
```

### 09:24:45 thinking

Now let me read the cycle 0003 variants to understand what they look like:

### 09:24:45 chat

Now let me read the individual cycle 0003 variant overlays and the baseline:

### 09:24:46 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v*\\insight.md"
}
```

### 09:24:47 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\baseline\\**\\*"
}
```

### 09:24:48 thinking

It seems the variant directory doesn't have `insight.md` files directly. Let me look at the structure differently.

### 09:24:49 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\**\\*"
}
```

### 09:24:51 thinking

Now let me read the key overlay files for the cycle 0003 variants (especially v002 and v003, the top winners):

### 09:24:52 chat

Let me read the key variant files:

### 09:24:52 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v003\\attempts\\01\\overlay\\src\\shared\\eviction_policy.rs"
}
```

### 09:24:53 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v002\\attempts\\01\\overlay\\src\\shared\\eviction_policy.rs"
}
```

### 09:24:54 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v003\\insight.md"
}
```

### 09:24:54 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v002\\insight.md"
}
```

### 09:24:54 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v001\\insight.md"
}
```

### 09:24:57 thinking

Now let me read the segment files to understand the promotion/probing logic in detail, and the item struct:

### 09:24:57 chat

Now let me read the dash_segment implementation to understand the promotion mechanics:

### 09:24:59 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v003\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 09:24:59 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v003\\attempts\\01\\overlay\\src\\shared\\item.rs"
}
```

### 09:25:03 thinking

Now I have a clear picture of the codebase and evolution history. Let me also look at the baseline to understand what the baseline segment looks like and what files can be overlaid:

### 09:25:03 chat

Now let me look at the baseline segment to understand the full picture:

### 09:25:04 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\baseline\\src\\**\\*"
}
```

### 09:25:06 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\dash\\**\\*"
}
```

### 09:25:10 thinking

Let me look at the v002 and v003 in cycle 3 to understand the full context. I've already seen their `eviction_policy.rs` files, which are identical to each other, and their `dash_segment.rs` files. Let me check the v002 variant's `dash_segment.rs` and v001's to understand what they differ on:

### 09:25:10 chat

Let me read the v002 cycle 0003 dash_segment to understand how it differs from v003:

### 09:25:12 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v002\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 09:25:12 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v001\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 09:27:41 thinking

Now I have a thorough understanding of the codebase and the evolution history. Let me synthesize what I know and design 3 distinct hypotheses:

**Summary of what's known:**

1. **v003/cycle3 (top winner, score 0.185)**: MRU-end insertion + vacancy-gated frequency promotion (threshold ≥ 2, BUT only fires when probing has space). Hit rate = 0.6796. No regressions.

2. **v002/cycle3**: MRU-end insertion + frequency threshold ≥ 3 (no vacancy gate). 

Hit rate = 0.6788 but introduces a p99 latency regression of 300ns.

3. **v001/cycle3**: MRU-end insertion + stash-bypass when target is full. This causes hit rate regression because the stash bypass triggers too frequently under MRU-end insertion.

**Open directions from the cycle-3 insight:**
1. **Combine vacancy gate + threshold ≥ 3**: Gate fires only when probing has space AND counter ≥ 3. This could push hit-rate higher than either alone (v003 uses threshold ≥ 2 + vacancy gate).
2. **Vacancy-gated stash bypass**: Retry stash insertion only when probing has available space. Looking at the hit-rate rankings across the cycle winners, v002 actually has the highest at 0.6796, but I need to reconcile this with what the overlay files show—v002 uses a simpler threshold approach while v003 adds the vacancy gate on top of a lower threshold, yet v003's hit-rate is slightly lower at 0.6788. 3, no vacancy gate) achieves the highest hit-rate at 0.6796 but introduces a p99 latency regression, while v003 (threshold ≥ 2 with vacancy gate) trades a slight hit-rate dip to 0.6788 for zero latency regression. For cycle 4, I'm planning H01 to combine both approaches—raising the threshold to ≥ 3 while keeping the vacancy gate from v003, which should preserve the latency gains while potentially recovering the hit-rate. H02 explores a different angle with vacancy-gated stash bypass.

H03 introduces counter reset on promotion—when an item gets promoted from target to probing via the frequency gate, I'd reset its lfu_counter to zero so it starts fresh in the probing tier rather than carrying stale accumulated counts from its time in the target tier.

Now I'm reconsidering what H03 should actually be. The cycle asks for three distinct ideas, and I already have the vacancy-gate threshold combination and the stash bypass with vacancy guard. For H03, I'm exploring whether LFU eviction in the stash tier could work—evicting the least-frequently accessed item when stash overflows instead of using LRU, though I need to verify this hasn't already timed out in earlier cycles. eviction policy defaults to ClassicLRU, so I need to look at the eviction_policy module, the dash_segment implementation, and potentially the item struct to make changes. For H03, I'm considering a dynamic vacancy gate that adjusts its threshold based on probing load—using a lower threshold when there's more capacity and raising it when slots are scarce, so the system adapts to current pressure.

But there's a cleaner approach: resetting the LFU counter when an item gets demoted from probing back to target. The idea is that if an item was evicted from probing, it lost out to newer or hotter items, so it shouldn't retain its counter and unfairly trigger the vacancy gate again—it should start fresh in target and compete fairly with other residents there.

This is a minimal change in `demote_from_normal_bucket` in `dash_segment.rs`, just zeroing the counter before placing the item back in target. It's testable, directly targets hit-rate, and is genuinely novel.

Now I'm laying out the three hypotheses to test. The first raises the vacancy gate threshold from ≥ 2 to ≥ 3 in `promotion_get` to be more selective about which items get promoted, maintaining cascade-free behavior while potentially improving hit-rate. The second fixes the stash-bypass regression by adding a probing-vacancy guard so items from stash hits route directly to probing only when there's space available.

For the third hypothesis, I'm considering whether to reset the LFU counter when demoting items back into target from probing — this prevents evicted items from immediately re-qualifying for the frequency gate and keeps the signal meaningful. I'm weighing whether the parent should be cycle/0003/v003 or baseline, leaning toward v003 since that's where the frequency and vacancy gates are already established, then the change would be adding the counter reset in the demotion path when routing foreign residents back to target. For H01, I'm raising the vacancy-gate threshold from 2 to 3 in the promotion logic to be more selective about which items get promoted, which should improve hit rates on Zipfian workloads while keeping the cascade-prevention guarantee intact. H02 adds a probing-vacancy check to the stash-bypass condition so bypass only triggers when there's actually room in probing, recovering the performance gains without the regression. The idea here is that resetting the LFU counter when items get demoted back from probing to target keeps the frequency signal accurate in the target tier, preventing recently-evicted items from immediately re-qualifying for promotion. I'd modify `demote_from_normal_bucket` to set `item.lfu_counter = 0` before placing the item back into a normal bucket in both the foreign resident case and potentially the stash branch, and handle the cascade scenario when target fills up.

Actually, I'm second-guessing whether resetting on demotion to stash is necessary since stash residents are already cold anyway. The real win is resetting when demoted from probing back to target — that's what prevents items with accumulated counter values from immediately re-qualifying for the vacancy gate when they return to target. Let me trace through where `demote_from_normal_bucket` gets called: when a new item enters the probing bucket and evicts the LRU item, that evicted item gets demoted either to its true target bucket or directly to stash.

For foreign residents taking the one-more-chance path to target, I need to check how `lfu_counter` behaves. In v003, the counter only increments in `promotion_get` when an item is accessed in target as non-hottest, but items in probing don't increment it through `get_from_position` under ClassicLRU. So when an item gets evicted from probing via cascade from a hot item, it arrives at target with whatever counter it had.

This creates an interesting scenario: an item with `lfu_counter=2` from its previous time in target gets promoted to probing, then evicted back to target, and on the next access its counter hits 3 and immediately re-triggers the vacancy gate if probing has space. That's actually reasonable behavior since the item is being re-accessed. But if I reset the counter after demotion from probing, the item would need to accumulate accesses again before qualifying for re-promotion.

The tradeoff is clear: resetting the counter reduces churn and keeps steadier items in probing longer, but it might also slow down legitimate re-promotion of genuinely hot items that oscillate between the two regions. The real issue is that items accumulate in probing and then bounce back with high counters, creating unnecessary churn. I need to think about whether counter-reset actually solves this or just masks the underlying problem. a different approach where I check if the stash bucket has lower-priority items when target is full, allowing in-stash MRU promotion without cascade risk. Though that's getting complicated, so maybe I should explore something simpler instead — the `Swap` policy that gradually bubbles items toward MRU on each hit rather than jumping them immediately, giving frequently-accessed items a more natural promotion curve in target.

Now I'm trying to figure out where the eviction policy gets configured in the production code. I need to check `dash_settings.rs` to see what the baseline uses, but I'm realizing I don't have access to all the source files to trace this properly.

For H03, I'm thinking the approach should be: when a stash hit promotes an item back to the target tier, reset its lfu_counter to zero. The idea is that an item coming back from stash is essentially starting fresh, so it shouldn't carry old frequency information — this way the vacancy gate can make better decisions based only on recent activity in the target tier. I'd implement this in the stash-hit path of `promotion_get`, right after retrieving the data from the stash buckets.

The rationale is solid: items end up in stash either because they were evicted as LRU candidates or cascaded out during probing, so by the time they're accessed again and promoted back, their old counter is stale. Resetting it gives a cleaner recency-adjusted frequency signal, and this approach is distinct from the threshold and bypass changes while being directly testable.

I'll go with this for H03. Now let me finalize the full set of hypotheses.

For H01, I'm raising the vacancy-gate threshold from 2 to 3 hits to be more selective about which items get promoted through the frequency path—only those with genuinely strong reuse evidence will qualify, which should improve hit rate by avoiding noisy promotions.

For H02, I'm adding a probing-vacancy check to the stash-saturation bypass so that items only bypass target and go straight to probing when there's actually open capacity, preventing cascade evictions from stash misses.

For H03, I'm resetting the lfu_counter when an item moves from stash to target so that its frequency qualification in the target reflects only accesses that happen after promotion. These three strategies are genuinely distinct—one adjusts the frequency threshold itself, another introduces a new vacancy-gated bypass path, and the third ensures fresh frequency signals by resetting the counter on stash promotion. I could alternatively cite cycle/0003/v002 (which had threshold ≥ 3 without the vacancy gate) and add the gate to that version instead, but the current approach with all three hypotheses from v003 works since they target different levers within the same mechanism.

Another option is to base H03 on cycle/0002/v001, the combined MRU+timestamp winner from cycle 2, and introduce something entirely new—though that version lacks both the vacancy gate and frequency gate, so I'd need to either add those or try a different mechanism altogether. Let me explore whether changing the eviction policy itself (from ClassicLRU to Swap) on the target and probing buckets could improve hit rate by letting gradually-accumulated access patterns drive item positioning rather than single-access jumps.

The challenge is that the eviction policy gets configured through `DashSettings` from the JNI layer, and I'm not certain I can modify it just by changing `dash_segment.rs`—it depends on what settings are passed in. Since I can't see where the default policy is actually configured and any eviction policy changes would likely need to touch `DashSettings` or segment instantiation, this approach might be messier than I'd like.

I should probably stick with my original three hypotheses all parented on v003, and I'm now looking back at the cross-cycle winners to see which scores and hit rates I'm working with. best hit-rate at 0.6796. I'm realizing that adding a vacancy gate to v002 would produce the same code as raising the threshold in v003, so that would be redundant. Let me finalize three distinct hypotheses instead, starting with raising v003's threshold from ≥ 2 to ≥ 3.

H02 focuses on bypassing the stash for better routing, and H03 resets the frequency counter during promotion—all three are mechanically different experiments building on v003. Now I need to flesh these out with actual code references for the changes section and write them to the output directory.

### 09:27:41 chat

Now I have all the context I need. Let me check the output directory:

### 09:27:41 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\**\\*"
}
```

### 09:27:50 thinking

The directory exists (well, those files exist). The output path for the hypotheses is:
`C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0004\hypotheses`

Let me now write the 3 hypothesis files. Based on my analysis:

**H01** (parent: cycle/0003/v003): Raise vacancy-gate threshold from ≥ 2 to ≥ 3. The change is minimal (one line in `promotion_get`) and should make the frequency gate more selective while maintaining cascade-free behavior.
**H02** (parent: cycle/0003/v003): Vacancy-gated stash bypass. When stash hit finds full target, bypass to probing only when probing has space too. This addresses the cycle/0003/v001 failure by adding the structural vacancy guard.
**H03** (parent: cycle/0003/v003): Reset lfu_counter to 0 when an item is promoted from stash to target. This ensures the frequency signal in target only reflects post-promotion accesses, preventing stale counters from immediately re-triggering the vacancy gate.

Now I'll write these files.

### 09:27:53 chat

Now I have everything I need. Let me synthesize the context and write the three hypotheses.

**Key findings:**
- **cycle/0003/v003** (top winner): MRU-end insertion + vacancy gate (threshold ≥ 2, fires only when probing has space → zero cascade cost). Hit-rate 0.6788, clean latency.
- **cycle/0003/v002**: MRU-end insertion + threshold ≥ 3, no vacancy gate. Higher hit-rate (0.6796) but p99 regression to 300ns.
- **cycle/0003/v001**: Stash bypass when target is full. Regressed because bypass fired too often under MRU-end insertion without a vacancy guard.

**Cycle-4 strategy:** H01 = combine v003's vacancy gate with ≥ 3 threshold; H02 = vacancy-gated stash bypass (fix the v001 failure structurally); H03 = reset lfu_counter on stash-to-target promotion to keep frequency signal fresh.

### 09:28:06 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\hypotheses\\H01.md",
  "content": "# H01\n\nRaising the vacancy-gate frequency threshold from ≥ 2 to ≥ 3 in `promotion_get` will improve hit rate by promoting only items with stronger reuse evidence while preserving the cascade-free guarantee that v003's vacancy check already provides.\n\n## Parent\n\ncycle/0003/v003\n\n## Changes\n\nIn `src/dash/segments/dash_segment.rs`, inside `promotion_get`, locate the non-hottest target branch (the `else` arm where `is_hottest` is `false`). The current condition for the vacancy-gated frequency promotion is:\n\n```rust\nif counter >= 2 && probing_has_space {\n```\n\nChange it to:\n\n```rust\nif counter >= 3 && probing_has_space {\n```\n\nThat is the only logic change. The `probing_has_space` vacancy guard remains in place exactly as in v003, preserving the structural guarantee that gate-fires never trigger cascade evictions. The `lfu_counter` increment immediately before (`core.buckets[target_idx].get_items_mut()[item_idx].lfu_counter += 1`) is unchanged.\n\nUpdate the module-level doc comment and `promotion_get`'s inline comment to reflect the new threshold value (replace \"2\" with \"3\" wherever the threshold is described). Update the `vacancy_gated_frequency_promote_fires_when_probing_has_space` test to pre-set `lfu_counter = 2` (so the next hit reaches 3) and adjust the `vacancy_gated_promote_does_not_fire_when_probing_full` test's pre-set to `lfu_counter = 2` as well. Add a new test `vacancy_gated_frequency_promote_requires_three_hits` that verifies a non-hottest item with `lfu_counter = 2` is NOT yet promoted (gate stays silent at threshold 2) but IS promoted after one more hit brings the counter to 3.\n\nNo changes to `src/shared/item.rs`, `src/shared/eviction_policy.rs`, or any other file.\n\n## Rationale\n\ncycle/0003/v002 demonstrated that threshold ≥ 3 (without the vacancy guard) produces the highest raw hit-rate (0.6796) seen across all cycles, but at the cost of a p99 latency regression. cycle/0003/v003 showed that the vacancy guard is sufficient to eliminate cascade latency entirely, even at the lower threshold of ≥ 2. Combining the guard from v003 with the higher threshold from v002 should preserve the zero-cascade cost while matching or exceeding v002's hit-rate selectivity, since the gate now fires on a stricter subset of items with genuinely repeated reuse.\n"
}
```

### 09:28:24 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\hypotheses\\H02.md",
  "content": "# H02\n\nAdding a probing-vacancy guard to the stash-saturation bypass will improve hit rate by routing stash-promoted items directly into probing only when a slot is available, eliminating the cascade eviction that made the unguarded bypass regress in cycle/0003/v001.\n\n## Parent\n\ncycle/0003/v003\n\n## Changes\n\nIn `src/dash/segments/dash_segment.rs`, inside the stash-hit branch of `promotion_get` (the `if let Some(item_idx) = core.stash_buckets[stash_idx].get_position(key)` arm), add a dual-condition bypass path between the existing `remove_at` call and the `promote_to_target` call.\n\nCurrent code in that branch (after removing the item from stash):\n\n```rust\nlet data = core.stash_buckets[stash_idx].remove_at(item_idx);\npromote_to_target(core, data, insert_pos);\n\nlet Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {\n    unreachable!(...)\n};\nreturn Some(&core.buckets[target_idx].get_items()[item_new_idx]);\n```\n\nReplace with:\n\n```rust\nlet data = core.stash_buckets[stash_idx].remove_at(item_idx);\nlet target_full = core.buckets[target_idx].is_full();\nlet probing_has_space = !core.buckets[probing_idx].is_full();\nif target_full && probing_has_space {\n    // Vacancy-gated stash bypass: target is saturated but probing has room.\n    // Land directly in probing with zero cascade cost.\n    debug!(\"promotion_get: key={key:?} stash bypass -> probing={probing_idx} (target full, probing has space)\");\n    promote_to_probing(core, data, insert_pos);\n    let item_new_idx = core.buckets[probing_idx]\n        .get_position(key)\n        .expect(\"promote_to_probing must leave key in probing (vacancy-gated stash bypass)\");\n    return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);\n} else {\n    // Normal path: target has capacity, or both tiers are full (must cascade).\n    promote_to_target(core, data, insert_pos);\n    let Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {\n        unreachable!(\"promote_to_target must leave key in target\")\n    };\n    return Some(&core.buckets[target_idx].get_items()[item_new_idx]);\n}\n```\n\nUpdate the struct-level and `promotion_get` doc comments to describe the vacancy-gated bypass. Add a unit test `stash_bypass_fires_only_when_target_full_and_probing_has_space` that: (a) fills target, leaves probing with one vacancy, does a stash hit, and asserts the item lands in probing; (b) fills both tiers, does a stash hit, and asserts the item goes through target as before. No changes to any other file.\n\n## Rationale\n\ncycle/0003/v001 showed that stash bypass without a vacancy guard regresses under MRU-end insertion because back-insertion keeps target saturated most of the time, causing the bypass to fire far too often and turning probing into a spillover sink. The vacancy-gated frequency promotion in v003 proved that tying any upward-tier move to probing capacity structurally eliminates cascade cost. Applying the same guard to the stash bypass creates the analogous cascade-safe path: bypass fires only when the receiving tier has room, so no eviction can occur from that insertion. On traces where stash hits are common and probing vacancies exist at the time of those hits, this should recover hit rate without latency regression.\n"
}
```

### 09:28:42 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0004\\hypotheses\\H03.md",
  "content": "# H03\n\nResetting the lfu_counter to zero when an item is promoted from stash into its target bucket will improve hit rate by ensuring the vacancy-gate threshold is evaluated against only fresh post-promotion accesses, preventing stale pre-demotion counts from triggering premature frequency promotion.\n\n## Parent\n\ncycle/0003/v003\n\n## Changes\n\nIn `src/dash/segments/dash_segment.rs`, inside the stash-hit branch of `promotion_get`, reset `lfu_counter` to zero on the item before it is placed back into the target tier. Specifically, immediately after:\n\n```rust\nlet data = core.stash_buckets[stash_idx].remove_at(item_idx);\n```\n\nadd:\n\n```rust\nlet mut data = core.stash_buckets[stash_idx].remove_at(item_idx);\ndata.lfu_counter = 0;\n```\n\n(The variable binding must be `mut` to allow field assignment; change the existing `let data` to `let mut data`.)\n\nThe subsequent `promote_to_target(core, data, insert_pos)` call is unchanged; the reset item arrives in target with a clean counter.\n\nNo changes to `demote_from_normal_bucket`, the non-hottest target branch, the probing branch, or any eviction policy. The vacancy gate itself (`if counter >= 2 && probing_has_space`) remains exactly as in v003; only the counter state carried into target from stash is affected.\n\nAdd a unit test `stash_promoted_item_enters_target_with_zero_counter` that: puts an item, manually pre-sets its stash bucket's `lfu_counter` to a non-zero value, performs a stash-hit `get`, then reads back the item from its target bucket and asserts `lfu_counter == 0`. Also add a test `stash_promoted_item_must_re_earn_vacancy_gate` that verifies a stash-promoted item (counter reset to 0) is not vacancy-gate promoted on the first target hit (counter becomes 1, gate stays silent) but is promoted on the second target hit (counter reaches 2, gate fires if probing has space).\n\nNo changes to `src/shared/item.rs` or any other file.\n\n## Rationale\n\nUnder v003, an item may accumulate `lfu_counter` hits while in its target bucket, earn a vacancy-gate promotion to probing, then be cascade-evicted back to target (via `demote_from_normal_bucket`) or eventually demoted to stash. When it reaches stash and is later re-accessed (a stash hit), it carries its old `lfu_counter` forward. Upon re-entering target at the MRU end (Back insertion), its counter is already ≥ 2, so a single additional target hit immediately re-triggers the vacancy gate. This can create rapid oscillation between target and probing for items whose counter was accumulated long before their stash demotion — a frequency signal that may no longer reflect current access patterns. Resetting the counter at the stash-to-target boundary forces each re-entry to the target tier to re-earn promotion eligibility from fresh evidence, making the frequency signal strictly post-promotion and potentially increasing the hit-rate accuracy of vacancy-gate promotions.\n"
}
```

### 09:28:52 chat

All three hypotheses have been written to `cycles/0004/hypotheses/`. Here is a summary:

---

**H01 — Raise vacancy-gate threshold to ≥ 3** (parent: `cycle/0003/v003`)
In `promotion_get`, change `if counter >= 2 && probing_has_space` to `if counter >= 3 && probing_has_space`. This is a single-line edit that combines v002's higher selectivity (which produced the best raw hit-rate, 0.6796) with v003's structural cascade guarantee. The vacancy guard stays in place, so no p99 regression can occur from this path.

**H02 — Vacancy-gated stash bypass** (parent: `cycle/0003/v003`)
In the stash-hit branch of `promotion_get`, add a dual-condition bypass: route the item directly to `promote_to_probing` only when *both* `target_idx` is full *and* `probing_idx` has at least one vacancy. Fall back to `promote_to_target` otherwise (target not full, or both tiers full). This is the structural fix for cycle/0003/v001's failure — the unguarded bypass fired too often under MRU-end insertion; the probing-vacancy check makes it cascade-safe.

**H03 — Reset lfu_counter on stash-to-target promotion** (parent: `cycle/0003/v003`)
In the stash-hit branch of `promotion_get`, after removing the item from stash, set `data.lfu_counter = 0` before calling `promote_to_target`. Items returning from stash may carry stale frequency counts that cause the vacancy gate to fire immediately on the very next target hit. Resetting forces re-accumulation of fresh post-promotion evidence, potentially making vacancy-gate promotions more accurate.


---

- finished: 2026-07-19T09:28:52+00:00
- duration: 259.92s
- status: ok
- input_tokens: 14
- output_tokens: 13793
- cache_read_tokens: 584465
- cache_write_tokens: 64787
- cost_usd: 0.626739
