# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/hypotheses
- allow_edits: True
- started: 2026-07-04T21:25:22+00:00

## timeline

### 21:25:25 thinking

Let me read all the necessary context files to understand the current state of the research and generate well-informed hypotheses for cycle 0010.

### 21:25:26 tool: Bash

```json
{
  "command": "ls /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/variants/",
  "description": "List cycle 0009 variants"
}
```

### 21:25:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/cycle-winners.json"
}
```

### 21:25:27 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/cycle-insight.md"
}
```

### 21:25:27 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/leaderboard.md"
}
```

### 21:25:28 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cross-cycle-insights.md"
}
```

### 21:25:31 thinking

Now let me read the cycle 0009 variants and the baseline, plus the cross-cycle history.

### 21:25:32 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/variants/v001/insight.md"
}
```

### 21:25:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/variants/v002/insight.md"
}
```

### 21:25:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/variants/v003/insight.md"
}
```

### 21:25:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles-ledger.jsonl"
}
```

### 21:25:37 thinking

Now let me read the actual source code for the baseline and the cycle 0008/v003 winner variant, so I understand exactly what the code looks like and can write precise, implementable hypotheses.

### 21:25:38 chat

Now let me read the baseline source code and the cycle 0008/v003 overlay to understand the actual implementation:

### 21:25:39 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/inputs/subjects/associative_cache/baseline"
}
```

### 21:25:39 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0008/variants/v003"
}
```

### 21:25:43 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0008/variants/v003/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 21:25:43 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0008/variants/v003/attempts/01/overlay/src/shared/eviction_policy.rs"
}
```

### 21:25:44 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0008/variants/v003/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 21:25:47 thinking

Now I have a thorough understanding of the codebase and research history. Let me also look at the baseline source and the cycle 0009 variant overlays to get a complete picture.

### 21:25:47 chat

Now let me look at cycle 0009 v001 and v003 overlays to understand what was tested:

### 21:25:49 tool: Glob

```json
{
  "pattern": "**/overlay/**",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/variants/v001"
}
```

### 21:25:49 tool: Glob

```json
{
  "pattern": "**/overlay/**",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/variants/v003"
}
```

### 21:25:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/variants/v001/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 21:25:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/variants/v003/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 21:28:55 thinking

Now I have a thorough understanding of the codebase and history. Let me now plan the three hypotheses for cycle 0010.

Key constraints:
1. At least one hypothesis MUST cite `cycle/0008/v003` (the incumbent cross-cycle winner with hit-rate=0.7298)
2. Three distinct strategies
3. No position-split decay (proven inert)
4. No LFU warm-fallback after decay (proven harmful)
5. No ARC ghost re-admission (proven harmful)
6. No counter-ceiling tuning (proven inert)
7. No further geometry widening (proven saturated)
8. No CLOCK (proven inferior)

The cross-cycle insights and cycle 0009 recommendations point toward three promising directions: a full latency audit comparing the accumulated stack against simpler alternatives, counter-value-based asymmetric decay that applies different rates depending on whether the counter is above or below a threshold, and adjusting the SLRU segment ratio from 50/50 to 25/75 to better preserve hot entries.

I'm designing three hypotheses to test these ideas, starting with counter-value-based asymmetric decay, which the cycle 0009 analysis explicitly flagged as potentially unlocking gains that position-split decay couldn't deliver since counter value already captures the signal we need.

The second hypothesis is a protected-heavy 25/75 SLRU split instead of the current 50/50, which would give more of the hot working set permanent protection by adjusting the capacity allocation between probationary and protected tiers.

The third is establishing a baseline variant for the latency audit that's been deferred across multiple cycles, which will give us concrete performance data to compare against these optimization attempts.

Looking at the leaderboard, I notice the baseline shows per-op-time of 69, but the evolved variants don't have per-op-time measurements recorded even though the evaluator appears to be configured. The fitness vectors only capture hit-rate, so the latency data isn't being propagated through the variant tracking system.

Since I'm working within the constraints of overlay files and can't modify the evaluation configuration, I should focus on hit-rate improvements. The open questions from cycle 0009 specifically recommend the counter-value asymmetric decay approach from v003, which uses different decay rates based on counter thresholds in the warm-lock path. There's also the SLRU 25/75 segment split variant to consider.

For frequency-biased eviction, I'm reconsidering whether scanning for the global minimum-counter item makes sense given that LFU warm-fallback after decay has proven harmful in prior cycles. I'm also thinking about whether tightening the cold threshold—moving from counter==0 to counter<=1 for eviction—might help, though cycle 0003 already explored this direction.

Looking at what's actually worth trying: counter-value asymmetric decay and the 25/75 SLRU split are both promising. Back-insertion for new items regressed before, so I'll skip that. What's interesting is triggering decay earlier—not waiting until all items are warm, but preemptively decaying when only a fraction are cold—to avoid getting locked into warm states. I'm also considering whether reversing the scan direction, checking from the newest items backward instead of oldest forward, could help avoid evicting recently-inserted items.

Magnitude-3 decay is worth exploring since magnitude-2 outperformed magnitude-1, though going too aggressive might destabilize the cold pool. The real novelty is proactive decay—triggering it when the cold pool drops below a threshold like 25% rather than waiting for a warm-lock crisis. Let me focus on three distinct approaches to test.

For H01, I'm modifying the decay logic to be counter-aware: items with lower counters (1-2) get hit harder with magnitude-2 decay to collapse them to cold quickly, while higher-counter items (3+) only decay by 1 to preserve their standing. For H02, I'm shifting the SLRU split from 50/50 to 25/75, giving the protected tier much more capacity to retain warmer items longer before they're vulnerable to eviction.

H03 takes a different angle—instead of waiting until the cold pool is completely exhausted before triggering decay, I'm proactively replenishing it by firing decay whenever cold items drop below a threshold like max_size / 8. This keeps the cold pool stocked and ready without letting warm-lock starve it entirely.

I'm also noticing the cycle notes mention a deferred latency audit that should run next, and I'm seeing per-op-time metrics in the leaderboard that I should factor into my variant design. about cycle 0008/v003 with its wide buckets, cold-first eviction, mid-point insertion, and bulk decay by 2 under a counter cap of 4. The cycle 0009 insight recommends exploring counter-value-based asymmetric decay instead of position-split approaches, and this remains untested across cycles.

Now I'm drafting H01, which proposes replacing the uniform magnitude-2 decay with value-split decay—magnitude 2 for counters ≤2 and magnitude 1 for counters ≥3—to concentrate cold-pool replenishment on lukewarm items while protecting the hot tier.

For H02, I'm shifting the SLRU segment split from 50/50 to 25/75 (probationary/protected) to give the confirmed working set more residency budget.  The change is in `prob_cap()` — dividing `self.max_size` by 4 instead of 2, with `prot_cap()` automatically yielding 75% protected capacity. The rest of the SLRU mechanics (cold-first scan, decay-by-2 warm-lock, promotion on hit) stay the same.

For H03, I'm adding proactive bulk counter decay triggered when cold-pool occupancy drops below a low-water mark like `max_size / 8`, rather than waiting for a full warm-lock event, to reduce warm-lock frequency.

Modifying `evict_item()` in `associative_cache_bucket.rs` to count cold items first, then apply decay-by-2 to all items if the cold count falls below the threshold or hits zero, ensuring the cold pool refreshes before it empties completely and avoiding the temporary hit-rate dip.

Actually, I'm reconsidering whether decaying when there are still some cold items is too aggressive—a low-water mark of `max_size / 8` would trigger decay fairly often during normal operation and might prematurely convert warm items to cold. The real issue is the hiccup when warm-lock kicks in, so maybe the threshold needs to be tighter or the approach needs refinement. so the issue is that after decay, index 0 might still be warm if it started with a high counter like 3 or 4. A better fix would be to scan for the first cold item after decay rather than blindly removing index 0, ensuring we're actually evicting a cold entry. After decay, items with low counters (1 or 2) are already likely to be cold, so rescanning would probably just pick index 0 again anyway. I'm considering a different approach instead — what if the cold-first scan only checks the oldest quarter of the bucket rather than the entire bucket? This would be faster while still being effective since older items are more likely to be cold, but I need to think through whether this actually targets the right problem.

I'm also exploring proactive decay triggered on every K-th put operation rather than only during eviction, similar to CLOCK but less aggressive, though CLOCK itself didn't work well in earlier testing. Stochastic cold-first eviction with some probability of picking a random cold item instead of the first one could break strict FIFO ordering, but that seems unlikely to help either.

Looking back at the open questions from previous cycles, I've already addressed counter-value-based asymmetric decay and SLRU segment ratio tuning. The remaining structural alternatives I haven't tested yet include giving items a second chance before eviction like CLOCK-Pro, or adjusting where items are inserted based on their counter estimate rather than always using the midpoint.

For the two-chance eviction approach, I'd decrement the counter on first eviction attempt and only evict if it reaches zero, but this runs into the same problem CLOCK had—continuous sweeps erode protection for active items. A frequency-stratified scan that looks for the lowest counter value rather than just zero items might work better when the cold pool is depleted.

I'm realizing the current cold-first scan is O(n) and just grabs the first counter-0 item, but finding the best cold item to evict more efficiently could be the real win here. The leaderboard is stuck at 0.7298, and the open questions keep pointing back to counter decay asymmetry and SLRU ratio tuning as the likely next improvements.

So I'll go with the two recommended experiments (H01 and H02) and pick a third that's structurally different. For H03, I'm thinking about pushing the decay magnitude ladder further—if magnitude-2 beat magnitude-1 in cycle 0008, what happens with magnitude-3? That would make items at counters 1, 2, or 3 all drop to cold in one pass, aggressively replenishing the cold pool. The tradeoff is that genuinely medium-hot items at counter 3 might get evicted before they're re-accessed, potentially hurting hit rate, but it's worth testing since it's the next untested step in the magnitude progression and distinct from the value-split approach of H01.

For H03, I'm modifying the warm-lock decay logic to subtract 3 instead of 2 from the LFU counter, so only items accessed 4+ times stay warm after each decay event. This is straightforward to implement and directly tests whether more aggressive decay improves hit rate by expanding the cold pool faster.

The trade-off is clear: decay-by-3 will cool down counter-3 items that might still be active, whereas decay-by-2 keeps them warm. The experiment will show whether the benefit of reducing warm-lock recurrence outweighs the cost of prematurely demoting useful items.

I'll go with H03 as a natural next step in the decay magnitude exploration. Let me lock in my three hypotheses: H01 for counter-value asymmetric decay, H02 for SLRU with a 25/75 protected-heavy split, and H03 for bulk decay magnitude 3.

Now I need to verify the parent citations—the cross-cycle winner is cycle/0008/v003, so both H01 and H03 can reference that, while H02 can cite cycle/0009/v003 as a prior non-winner. Beyond these top candidates, I'm also considering whether there's a structurally different policy approach that could break through the hit-rate ceiling we're seeing on workload-a.

The research insights point toward exploring something fundamentally different from single-queue families—like LIRS-proper or an adaptive two-queue system—but those have either been attempted or are too complex. However, there's an interesting angle around per-op latency: the baseline fitness evaluator already tracks per-op-time metrics, so I could propose a simplified variant that trades a small hit-rate penalty for better latency performance by using a pure-LFU wide-bucket approach instead of the full stack mechanism.

The challenge is that I can't directly enable the per-op-time evaluator to measure improvements, and the cycle notes recommend a latency audit that isn't actionable from my position. So instead, I'm exploring a more creative direction: what if insertion points in the cache varied proportionally to capacity rather than using a fixed midpoint?

The issue with frequency-based insertion is that new items all start with a counter of zero—there's no way to distinguish them from returning items at insertion time since the key must be new to pass the initial position check. I'm considering whether inserting at a different fixed point, like one-third of the bucket length instead of the midpoint, might make items slightly colder and reduce congestion in the middle of the bucket.

But I'm realizing this approach is getting too incremental. A more interesting direction might be to flip the cold-first scan—instead of always scanning from the oldest items at index zero, what if I scan backwards from the end to find cold items there? Though with mid-point insertion and LRU promotion moving hot items to the back, the ordering gets complex: the oldest untouched items cluster at the front, recently inserted items sit around the midpoint, and the back fills with frequently accessed items.

Actually, let me explore something structurally different—what if instead of decaying all counters by the same fixed amount, I decay each counter proportionally to its current value? So a counter of 4 might halve to 2, while a counter of 1 drops to 0, creating a frequency-weighted decay that preserves the relative differences between hot and cold items. I'm listing out the distinct strategies I haven't fully explored yet: varying the insertion point in the SLRU structure, testing a decay magnitude of 3 instead of 2, experimenting with a cold-pool scan that only covers a limited window rather than the full pool, and tracking hit-count separately from recency-position using a two-tier counter system. The most promising angles seem to be pushing the decay magnitude higher as a natural parametric step, or triggering decay preemptively based on cold-pool thresholds rather than waiting for it to happen reactively. I'm also recalling something from the cycle analysis about how min-counter LFU approximates the full behavior on wide buckets.

Now I'm wondering if I can combine the cold-first scan with a secondary frequency-based selection. Instead of always picking index 0 in the warm-lock phase, what if I picked the item with the globally minimum counter after applying the decay-by-2 operation? That would shift some items back to cold status, and the question becomes whether picking by minimum counter rather than by position would improve hit-rate. So the real issue is that after decay, items with counter 3 drop to 1 (still warm) while items with counter 1 or 2 become 0 (cold). The current approach evicts index 0 regardless, but if that item happens to have counter 3 and there's a cold item elsewhere in the middle, we're evicting the wrong thing. A post-decay scan for the first cold item would catch this edge case and be a meaningful improvement. The issue is that by the time we reach the warm-lock eviction path, any item that was already cold before decay would've been caught and removed in the earlier scan. So the real benefit of the cold-first rescan only applies when an item *becomes* cold due to the decay operation itself—meaning it had a counter of 1 or 2 before the saturating subtraction. But the item at index 0, being the oldest and least recently accessed, might still have a higher counter value, making it unclear whether the rescan would actually find a newly-cold item or just evict the same LRU front candidate anyway. counter 0 (cold). So the re-scan would catch it as a cold item to evict, which is the right target. The improvement is that by scanning from the middle outward, we're more likely to find recently-accessed items with low counters rather than old items that happen to have high counters at the front.

I'm settling on three hypotheses to test: asymmetric decay where low counters decay faster than high ones, shifting the SLRU protected segment from 50/50 to 25/75, and adding a post-decay cold re-scan in the warm-lock path to catch items that dropped to zero after decay. Refining the decay logic...

For the SLRU cache bucket, I'm adjusting the probationary and protected capacity calculations so the probationary segment takes up 25% of the max size instead of 50%, with a minimum of 1 to avoid edge cases.

For H03, I'm modifying the eviction logic to apply a decay-by-2 reduction to all items' LFU counters during the warm-lock phase, then evict whichever item ends up at the front of the LRU order after that adjustment. Now I'm ready to implement these approaches, but first I should verify that the output directory is set up correctly.

### 21:28:55 chat

Now I have all the context I need. Let me check the output directory and write the three hypothesis files:

### 21:28:56 tool: Glob

```json
{
  "pattern": "**",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010"
}
```

### 21:28:59 thinking

The hypotheses directory doesn't exist yet. I need to create the files at:
/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/hypotheses/H01.md
/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/hypotheses/H02.md
/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/hypotheses/H03.md
Now I'll write all three hypothesis files.

### 21:28:59 chat

The hypotheses directory needs to be created. Now I'll write all three hypothesis files:

### 21:29:14 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/hypotheses/H01.md",
  "content": "# H01\n\nReplacing uniform magnitude-2 decay with value-split decay (magnitude 2 for counter ≤ 2, magnitude 1 for counter ≥ 3) in the warm-lock path will raise hit-rate by concentrating cold-pool replenishment on the lukewarm tier while preserving the hot tier's frequency signal.\n\n## Parent\n\ncycle/0008/v003\n\n## Changes\n\nIn `evict_item()` in `src/associative_cache/associative_cache_bucket.rs`, replace the uniform\nwarm-lock decay loop with a value-split version:\n\nCurrent code (Step 2):\n```rust\nfor item in self.items.iter_mut() {\n    item.lfu_counter = item.lfu_counter.saturating_sub(2);\n}\nSome(self.remove_at(0))\n```\n\nNew code (Step 2):\n```rust\nfor item in self.items.iter_mut() {\n    if item.lfu_counter <= 2 {\n        // Lukewarm tier (counter 1 or 2): decay aggressively — both become cold (0)\n        item.lfu_counter = item.lfu_counter.saturating_sub(2);\n    } else {\n        // Hot tier (counter 3 or 4): decay conservatively — preserve most of the signal\n        // counter 3 → 2 (warm), counter 4 → 3 (warm)\n        item.lfu_counter = item.lfu_counter.saturating_sub(1);\n    }\n}\nSome(self.remove_at(0))\n```\n\nThe rationale: cycle 0009 confirmed that position is not a reliable proxy for \"genuinely hot\"\nunder cap-4, but counter value is. This change honors that finding by decaying the lukewarm\ntier (counter 1–2) at full magnitude-2 (collapsing it to cold in one pass) while reducing the\ndecay cost on the hot tier (counter 3–4), leaving hot items at counter 2–3 rather than 1–2.\nThe net effect is a larger cold pool after each warm-lock event alongside a better-preserved\nfrequency gradient above it.\n\nAll other logic is unchanged: `eviction_policy.rs` keeps the counter cap at 4 and\n`get_from_position` promotion logic; `geometry.rs` keeps the factor-4 wide-bucket layout;\nthe pre-decay cold-first scan (Step 1) and the post-decay `remove_at(0)` fallback remain as-is.\n\n## Rationale\n\nCycle 0009 v001 demonstrated that position-split decay is inert because upper-half items span\nthe full counter range 1–4, making position an unreliable discriminator. The cycle 0009 insight\nexplicitly flags value-split decay as the next logical axis: \"a value-split may preserve the hot\ntier without compressing its frequency signal.\" This hypothesis directly tests that conjecture.\nUnder the current uniform magnitude-2 decay, a counter-3 item (accessed three times since the\nlast decay) is treated identically to a counter-1 item (accessed once), collapsing both to 0\nand 1 respectively — but the counter-3 item is meaningfully hotter. The value-split keeps\ncounter-3 → 2 and counter-4 → 3, making these items harder to evict immediately after a\nwarm-lock recovery, which should reduce spurious hot-item evictions on the next eviction call.\n"
}
```

### 21:29:30 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/hypotheses/H02.md",
  "content": "# H02\n\nChanging the SLRU probationary/protected segment split from 50/50 to 25/75 will raise hit-rate by giving the confirmed hot working set a larger residency budget per bucket, reducing displacement of protected items by one-hit wonders.\n\n## Parent\n\ncycle/0009/v003\n\n## Changes\n\nIn `src/associative_cache/associative_cache_bucket.rs` (the SLRU bucket implementation from\ncycle/0009/v003), change `prob_cap()` to allocate only 25% of `max_size` to the probationary\nsegment instead of 50%:\n\nCurrent code:\n```rust\nfn prob_cap(&self) -> usize {\n    self.max_size / 2\n}\nfn prot_cap(&self) -> usize {\n    self.max_size - self.prob_cap()\n}\n```\n\nNew code:\n```rust\nfn prob_cap(&self) -> usize {\n    // 25% probationary; at least 1 slot to keep the tier active.\n    (self.max_size / 4).max(1)\n}\nfn prot_cap(&self) -> usize {\n    // 75% protected — the remaining capacity after the probationary reserve.\n    self.max_size - self.prob_cap()\n}\n```\n\nFor a 64-slot bucket (factor-4 wide geometry with default 2048 capacity):\n- Before: 32 probationary + 32 protected\n- After:  16 probationary + 48 protected\n\nAll other logic from cycle/0009/v003 is preserved unchanged:\n- Probationary insertion at the back of the probationary segment.\n- Promotion from probationary to protected on the first GET hit (`get_from_position`).\n- Demotion of the oldest protected item to probationary-front when protected is full.\n- Cold-first eviction scan across probationary only (Step 1 of `evict_item`).\n- Decay-by-2 warm-lock confined to probationary (Step 2 of `evict_item`).\n- Fallback eviction of the oldest probationary item, then oldest protected if probationary is empty.\n- Wide-bucket factor-4 geometry from `geometry.rs`.\n\nThe `prob_count` field semantics are unchanged; the only behavioral difference is how\n`prob_cap()` and `prot_cap()` constrain the segment boundaries.\n\n## Rationale\n\nCycle 0009 explicitly identifies the 50/50 split as the top open question for the SLRU branch:\n\"What proportion of SLRU's deficit is attributable to the 50/50 segment split? A protected-heavy\nsplit (e.g., 25/75) might retain more of the working set per bucket.\" With a 50/50 split and\nwide (64-slot) buckets, the protected tier holds only 32 items per bucket. If the hot working\nset per bucket exceeds 32 items, legitimate hot items are demoted back to probationary and\nbecome eviction candidates alongside true one-hit wonders. A 75% protected allocation raises\nthe hot-item retention budget to 48 slots per bucket, reducing unnecessary demotion of the\nworking set. The probationary filter still exists at 25% (16 slots per wide bucket) — enough\nto absorb typical scan bursts before they reach the protected tier.\n"
}
```

### 21:29:48 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/hypotheses/H03.md",
  "content": "# H03\n\nAdding a post-decay cold re-scan in the warm-lock eviction path (instead of unconditionally removing index 0) will raise hit-rate by ensuring cold-first selection is honored even for items that become cold only after the decay pass.\n\n## Parent\n\ncycle/0008/v003\n\n## Changes\n\nIn `evict_item()` in `src/associative_cache/associative_cache_bucket.rs`, change the final\nstep of the warm-lock path to re-scan for the first cold item after applying the bulk\n`saturating_sub(2)` decay, rather than blindly removing the LRU front at index 0:\n\nCurrent code (Step 2, after decay loop):\n```rust\n// Evict the LRU front (index 0); it is now cold (or near-cold) after decay.\nSome(self.remove_at(0))\n```\n\nNew code (Step 2, after decay loop):\n```rust\n// Re-scan for the first cold item that the decay pass just created.\n// Items that had counter 1 or 2 are now cold (counter == 0).\n// Fall back to index 0 (LRU front) if none are cold — this can only happen\n// when all items had counter >= 3 before decay, i.e., all are now counter >= 1.\nlet post_decay_cold = self.items.iter().position(|item| item.lfu_counter == 0);\nSome(self.remove_at(post_decay_cold.unwrap_or(0)))\n```\n\nNo other changes: the pre-decay cold-first scan (Step 1), the decay loop magnitude\n(`saturating_sub(2)`), mid-point insertion in `put()`, the counter cap at 4 in\n`eviction_policy.rs`, and the factor-4 wide-bucket layout in `geometry.rs` all remain as-is.\n\nThe `unwrap_or(0)` fallback is sound: it fires only when all items had counter ≥ 3 before\ndecay (all now counter ≥ 1 after saturating_sub(2)), which is a valid warm state — removing\nthe LRU front (index 0, the oldest item) remains the correct tie-break.\n\n## Rationale\n\nUnder mid-point insertion combined with LRU promotion-to-back, index 0 holds the item that\nhas been in the bucket the longest without a GET hit — which makes it a good eviction candidate.\nHowever, there is no guarantee that index 0 has the lowest counter. An item inserted at the\nmid-point that was never accessed (counter 0) would already have been caught by Step 1's\npre-decay scan. But consider a scenario where: index 0 has counter 3 (it accumulated three\nhits since the last decay and was gradually pushed toward the front by subsequent\ninsertions/promotions); while an item at a middle index has counter 1 (inserted at mid-point\nand accessed once, but now aging). After decay-by-2, index 0 → counter 1 (still warm),\nmiddle item → counter 0 (cold). The current code evicts index 0 (warm, counter 1) instead of\nthe genuinely colder middle item. The post-decay re-scan corrects this by honoring cold-first\nsemantics even in the warm-lock recovery path, at the cost of one additional O(n) scan pass —\na cost that only fires during the rare warm-lock event, not on every eviction call.\n"
}
```

### 21:29:57 chat

All three hypothesis files are written. Here's a summary of what was produced for cycle 0010:

---

**H01** — *Counter-value asymmetric decay* (`cycle/0008/v003`)  
Replaces the uniform `saturating_sub(2)` warm-lock decay in `evict_item()` with a value-split: items at counter ≤ 2 (lukewarm) still decay by 2 (collapse to cold in one pass), while items at counter ≥ 3 (hot) decay by only 1 (preserve more frequency signal). Directly tests the conjecture from cycle 0009's open questions that value-based discrimination outperforms position-based discrimination under cap-4.

**H02** — *SLRU 25/75 protected-heavy split* (`cycle/0009/v003`)  
Changes `prob_cap()` from `max_size / 2` to `(max_size / 4).max(1)`, giving the protected tier 75% of each bucket instead of 50%. This addresses the top open question from cycle 0009: whether SLRU's 50/50 split is suppressing the working set below the hot-item threshold. All other SLRU logic (cold-first scan, decay-by-2 in probationary, wide geometry) is preserved.

**H03** — *Post-decay cold re-scan in warm-lock path* (`cycle/0008/v003`)  
Replaces the unconditional `remove_at(0)` after the bulk decay pass with a second cold-first scan (`position(|item| item.lfu_counter == 0).unwrap_or(0)`). This ensures cold-first eviction semantics are honored even for items that only become cold as a result of the decay — closing a subtle gap where the item at index 0 might have survived decay at counter 1 or 2 while a middle-index item became genuinely cold.


---

- finished: 2026-07-04T21:29:57+00:00
- duration: 275.43s
- status: ok
- input_tokens: 12
- output_tokens: 15983
- cache_read_tokens: 391432
- cache_write_tokens: 45825
- cost_usd: 0.530473
