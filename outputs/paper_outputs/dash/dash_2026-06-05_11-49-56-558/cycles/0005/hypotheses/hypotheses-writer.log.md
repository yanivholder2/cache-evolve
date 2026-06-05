# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/hypotheses
- allow_edits: True
- started: 2026-06-05T13:04:13+00:00

## timeline

### 13:04:16 thinking

Let me read all the relevant context files to understand the current state of the research and formulate good hypotheses for cycle 0005.

### 13:04:24 tool: Agent

```json
{
  "description": "Read all context files for cycle 0005 hypothesis generation",
  "subagent_type": "Explore",
  "prompt": "Read the following files and return their full contents. I need to synthesize hypotheses for a cache evolution research loop.\n\nFiles to read:\n\n1. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/leaderboard.md\n2. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles-ledger.jsonl\n3. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cross-cycle-insights.md (if it exists)\n4. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/cycle-winners.json\n5. /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/cycle-insight.md\n\nThen list the variant directories under /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/ and for each variant (v001, v002, v003, etc.) read:\n- The fitness-vector.json: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v<NNN>/eval/fitness-vector.json\n- The insight.md: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v<NNN>/insight.md\n- The overlay files (list what's in the overlay dir and read them): /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v<NNN>/attempts/ — find the latest attempt and read its overlay files.\n\nAlso read the baseline files:\n- /home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/dash_bucket.rs\n- /home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/segments/dash_segment.rs\n- /home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/eviction_policy.rs\n- /home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/geometry.rs\n- /home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/item.rs\n\nReturn all file contents in full."
}
```

### 13:04:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/leaderboard.md"
}
```

### 13:04:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles-ledger.jsonl"
}
```

### 13:04:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cross-cycle-insights.md"
}
```

### 13:04:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/cycle-winners.json"
}
```

### 13:04:27 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/cycle-insight.md"
}
```

### 13:04:27 tool: Bash

```json
{
  "command": "find /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/ -maxdepth 1 -type d -name \"v*\" | sort"
}
```

### 13:04:29 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v*/"
}
```

### 13:04:30 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/*"
}
```

### 13:04:32 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v001/eval/fitness-vector.json"
}
```

### 13:04:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v001/insight.md"
}
```

### 13:04:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v002/eval/fitness-vector.json"
}
```

### 13:04:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v002/insight.md"
}
```

### 13:04:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v003/eval/fitness-vector.json"
}
```

### 13:04:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v003/insight.md"
}
```

### 13:04:36 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 13:04:37 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v001/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 13:04:37 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v002/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 13:04:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v002/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 13:04:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v003/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 13:04:40 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/dash_bucket.rs"
}
```

### 13:04:40 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/segments/dash_segment.rs"
}
```

### 13:04:40 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/eviction_policy.rs"
}
```

### 13:04:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/geometry.rs"
}
```

### 13:04:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/item.rs"
}
```

### 13:04:42 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/**/*.rs"
}
```

### 13:04:44 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/**/*.rs"
}
```

### 13:04:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/dash/dash_bucket.rs"
}
```

### 13:04:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/dash/segments/dash_segment.rs"
}
```

### 13:04:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/eviction_policy.rs"
}
```

### 13:04:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/geometry.rs"
}
```

### 13:04:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/item.rs"
}
```

### 13:06:26 thinking

I'll wait for the agent to complete before proceeding.

### 13:06:27 chat

I've launched a research agent to gather all the context files. I'll wait for it to complete before writing the hypotheses.

### 13:06:34 thinking

The agent failed to read the files due to permission restrictions. Let me read the files directly using my own tools.

### 13:06:34 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants"
}
```

### 13:06:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/leaderboard.md"
}
```

### 13:06:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles-ledger.jsonl"
}
```

### 13:06:38 thinking

Let me read more context files - the cycle 0004 variants' fitness vectors, insights, and overlays.

### 13:06:39 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/cycle-winners.json"
}
```

### 13:06:40 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/cycle-insight.md"
}
```

### 13:06:40 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cross-cycle-insights.md"
}
```

### 13:06:43 thinking

Now let me read the cycle 0004 variants' fitness vectors, insights, and most importantly, the overlay files for the winner (v002) and the other variants.

### 13:06:44 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v001/insight.md"
}
```

### 13:06:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v002/insight.md"
}
```

### 13:06:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v003/insight.md"
}
```

### 13:06:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v002/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 13:06:50 thinking

Now let me read the other overlay files and the baseline source files to understand the complete picture.

### 13:06:50 chat

Now let me read the baseline source files and the other cycle 0004 overlays:

### 13:06:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/segments/dash_segment.rs"
}
```

### 13:06:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/geometry.rs"
}
```

### 13:06:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/shared/eviction_policy.rs"
}
```

### 13:06:54 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/inputs/subjects/dash/baseline"
}
```

### 13:06:57 thinking

The baseline doesn't have the source files in the baseline directory. Let me check the project structure to find the actual source files.

### 13:06:57 tool: Glob

```json
{
  "pattern": "**/*.rs",
  "path": "/home/user/cache-evolve/inputs"
}
```

### 13:06:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 13:06:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v001/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 13:07:02 thinking

Now let me look at the remaining overlay files for the cycle 0004 variants, to see what changes were made to v002 (winner) and v003.

### 13:07:02 chat

Now let me read the remaining cycle 0004 overlay files and the baseline geometry:

### 13:07:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v002/attempts/01/overlay/src/shared/geometry.rs"
}
```

### 13:07:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0004/variants/v003/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 13:07:05 tool: Glob

```json
{
  "pattern": "**/*.rs",
  "path": "/home/user/cache-evolve/inputs/subjects/dash/src"
}
```

### 13:07:08 thinking

Now let me read the remaining baseline source files to understand what I'm working with:

### 13:07:08 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/dash/segments/dash_segment.rs"
}
```

### 13:07:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/geometry.rs"
}
```

### 13:07:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/eviction_policy.rs"
}
```

### 13:07:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/item.rs"
}
```

### 13:08:03 thinking

Now I have a thorough understanding of the codebase. Let me also look at the bucket trait to understand `promotion_candidate_index` and `get_from_position` in more detail.

Let me now synthesize this to write three distinct hypotheses for cycle 0005.

## Summary of what I know:

### Current state:
- **Best hit-rate**: 0.7147 (cycle/0004/v002 = ghost re-admission + 75/25 topology)
- **Baseline**: 0.663
- **Workload**: workload-a (LIRS trace, recency-dominant, stable hot set)


- **Architecture**: The cache uses a promotion ladder where items move from stash through target to probing buckets on hits, with ClassicLRU eviction that moves accessed items to the back. The geometry splits buckets into normal and stash regions (currently 24/8), and items track frequency, timestamps, and other metadata for different eviction strategies.

- **Winning configurations**: The 24/8 split is confirmed optimal with no further topology gains available. Back-insertion on stash hits proved additive, and position-based recency via ClassicLRU is the dominant eviction signal. The current best variant uses a ghost set that re-admits evicted items at the front of the target bucket with near-zero overhead.

- **Next frontier**: Exploring how items are ordered within the normal tier itself to improve scan resistance.

- **Hypothesis 1 (building on cycle/0004/v002)**: Instead of inserting ghost re-admissions at the front, what if we place them at the back? This would treat re-admitted items as the most recently used, potentially preserving the latency gains we've seen. their target bucket, allowing them to promote to probing quickly. This is a concrete change to `promote_to_target` call in the ghost path of `put()` in `dash_segment.rs`.

Actually, looking at the code more carefully: in cycle/0004/v002's `put()`, when a ghost key is found, it removes the key and calls promote with `InsertPosition::Front`, which places the re-admitted item at the LRU end. The insight about neutralizing latency savings suggests flipping this to `InsertPosition::Back` would put the item at MRU instead, making it immediately eligible for promotion — worth testing.

For hypothesis 2, I'm considering intra-probing segmentation where items that get hit again within the probing tier get re-promoted to MRU, while newly-arrived items stay at the front. This creates finer-grained eviction ordering. But looking at cycle/0004/v002, `promotion_get` already calls `promote_to_probing` with `InsertPosition::Back`, so newly-promoted items are already landing at MRU.

The real issue is that hits on items already in probing don't trigger LRU movement — they're returned in-place. I could make probing a full LRU bucket by moving hit items to the back on access, preventing hot probing items from being evicted by cold newly-promoted ones. The current code just retrieves the item without reordering. So the eviction policy's `get_from_position` is already being called during probing hits, which means the LRU behavior within the normal tier is already happening. Let me explore whether using `Swap` as the eviction policy for the stash tier could help—it would gently bubble up frequently-accessed stash items toward MRU, potentially keeping them alive longer before they need to earn a second hit for promotion. But I need to check whether the eviction policy is applied per-tier or globally across all buckets in a segment.

Looking at how the eviction policy is used, it seems to be global across all tiers, and since ClassicLRU has been dominant in the cross-cycle history, switching policies would likely hurt performance. Instead, I'm thinking about the proportional re-entry position idea mentioned in the cross-cycle questions—when a stash hit gets promoted to target, rather than always inserting at the back, I could insert it at a position proportional to its access count, so items with higher hit counts land closer to MRU.

The challenge is that `promote_to_target` only accepts `InsertPosition::Front` or `InsertPosition::Back`, with no middle option. I'd need to either add a new variant to the bucket trait or find another way to implement this. But looking at what files I can actually modify—`dash_bucket.rs`, `dash_segment.rs`, `eviction_policy.rs`, `geometry.rs`, and `item.rs`—I can't change the trait definition in `bucket.rs` since it's not in the extension-point files.

Let me check what's in `dash_bucket.rs` since that's writable and might give me more flexibility for the bucket implementation.

### 13:08:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/dash/dash_bucket.rs"
}
```

### 13:08:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/src/shared/traits/bucket.rs"
}
```

### 13:18:15 thinking

Now I have a thorough understanding of the architecture. Let me synthesize the three hypotheses:

## Architecture understanding:

### Key facts:
1. **InsertPosition**: Only `Back` (MRU) and `Front` (LRU end)
2. **Promotion logic**: stash → target (via stash hit), target → probing (when hottest in target), probing (stays, LRU reordering on hit via ClassicLRU)
3. **cycle/0004/v002 (incumbent winner)**:
   - Geometry: 24 normal / 8 stash (75/25 split)
   - Ghost set: 32-entry FIFO tracking recently evicted-from-stash keys
   - Ghost re-admission: bypasses stash, inserts at `InsertPosition::Front` into target
   - Normal promotion: uses `InsertPosition::Back`
4. **Extension point files**: dash_segment.rs, geometry.rs, eviction_policy.rs, item.rs, dash_bucket.rs

### Open directions from cross-cycle insights:
1. "Intra-normal-tier eviction ordering: scan resistance or segmented LRU within the hot tier"
2. "Proportional re-entry position: insert at position proportional to access count"
3. "Ghost set refinements..."

### Exploring three hypotheses:

**H01 (parent: cycle/0004/v002)**: Ghost re-admission to `Back` instead of `Front`

The current approach inserts ghost re-admitted items at `InsertPosition::Front`, which forces them to climb from LRU to MRU before qualifying for promotion. The insight is that this negates most of the latency benefit the bypass was supposed to provide. Switching to `InsertPosition::Back` would place ghost re-admitted items directly at MRU, making them immediately eligible for promotion on their next access—this would make the ghost bypass genuinely valuable since a quickly re-arriving item could jump straight to probing without the intermediate climb. The change is localized to the ghost re-admission path in `dash_segment.rs`.

**H02 (parent: cycle/0004/v002)**: Intra-probing refresh behavior

Wait, I need to reconsider—probing already handles LRU reordering through its access path. Let me trace through the code more carefully. When an item is found in the probing bucket during a get operation, the system is already moving it to the back...

But stash hits are different. When a stash item is accessed through `promotion_get`, it gets promoted out of stash entirely and moved into the target bucket. The stash itself doesn't maintain LRU ordering for items that remain in it—only when the stash fills up and needs to evict. So the real question is what happens to stash items that get hit but aren't being promoted out.

Actually, looking at the code more carefully, every stash hit triggers an immediate promotion to target. There's no selective promotion based on how "hot" the item is within the stash. Then for items already in target, promotion to probing only happens if they're the hottest item in their target bucket (the MRU end for ClassicLRU). If they're accessed but not the hottest, ClassicLRU just moves them to the back within target.

Since v002 uses `InsertPosition::Back` when promoting from stash to target, newly arrived items start at MRU position, so they'll get promoted to probing on their very next access.

H02 proposes a two-tier stash admission strategy: instead of promoting every stash hit immediately, only promote items that have accumulated enough accesses in the stash itself (like `lfu_counter >= 1`). This way, items that only pass through stash once wouldn't get promoted, staying in stash longer.

But there's a catch—stash hits currently don't increment `lfu_counter` at all since items are removed immediately upon promotion. I could change that: track the counter in stash, so a first hit sets it to 1 and moves the item to MRU, then a second hit finally promotes to target. The concern is whether this adds too much friction to the promotion pipeline and creates new bottlenecks.

Actually, I'm noticing something interesting about LIRS—it distinguishes between items seen once versus multiple times, which is essentially what a stash LFU gate would do. But that approach was already tested in cycle 0001 with the `lfu_counter >= 2` gate on target→probing transitions, and it didn't work well. Though wait, that's different from gating stash→target directly.

Let me explore a different angle here.

When items get demoted from probing back to target, they currently land at the MRU position, making them immediate promotion candidates again—one more access sends them right back to probing. What if I changed that so demoted items arrive at the front of target instead, forcing them to compete harder before re-earning probing status? This would break the aggressive bounce cycle where items ping-pong between tiers without actually proving they're hot enough.

The tricky part is that the insertion position is currently a single variable threaded through the whole promotion-demotion chain, so I need to figure out how to differentiate insertion behavior based on where an item's coming from. operations. For cycle/0004/v002, `insert_pos = InsertPosition::Back`.

The cascade is: when a promotion evicts from probing:
- If the evicted item's target == source (true target is probing bucket), it goes to stash at `Back`
- If the evicted item's target != source (it's a "foreign" item in probing), it goes to its target bucket at `Back` (as one-more-chance)

Currently, "foreign" items arriving in target via the one-more-chance cascade land at `Back` (MRU). With ClassicLRU, this makes them immediately the promotion candidate, so one more hit promotes them again.

What if foreign demoted-from-probing items landed at `InsertPosition::Front` (LRU end of target)? This is a potentially interesting change: it would mean displaced foreign items have to earn their way back up in target before re-promoting. The challenge is that the current architecture uses the same `insert_pos` for both fresh entries and demotion cascades, so I'd need to split this logic in `demote_from_normal_bucket`.

Alternatively, there's a simpler angle: what if I increased the ghost set from 32 to 128 entries while also fixing the insertion position from `Front` to `Back`? That would combine two improvements at once, though I'd need to check whether H01 and H02 would interact in useful ways. be building on cycle/0004/v002 with ghost-set changes. That's near-duplicate territory. I should make them more distinct.

Let me reconsider the three hypotheses from fresh:

**Required**: At least one must cite `cycle/0004/v002`.

**H01 (parent: cycle/0004/v002)**: Ghost re-admission position change to Back (MRU)
- Changes: In the ghost re-admission path of `DashSegment::put()`, change `InsertPosition::Front` to `InsertPosition::Back` for the `promote_to_target` call.
- Testable: Does inserting ghost-admitted items at MRU (making them immediately promotion-eligible) raise hit-rate vs. the current Front insertion?
- Grounded: The cycle/0004 insight explicitly blamed `Front` insertion for neutralizing the ghost bypass benefit.

**H02 (parent: cycle/0003/v002 or baseline)**: Selective stash promotion based on LRU position
- What if stash hits only promote when the item is the most-recently-used in its bucket? Otherwise, apply an LRU move within the stash instead of promoting immediately.
- This changes `promotion_get` so that not every stash hit triggers promotion to target—only the hottest items in the stash bucket graduate.
- Risk: Could trap items in stash, but worth testing if it reduces unnecessary promotion churn.

**H03 (parent: baseline)**: Swap-like gradual LRU for stash tier
- Instead of ClassicLRU (move-to-back on every access), use a Swap-like policy where stash hits move an item one position toward MRU rather than all the way.
- Rationale: Reduces churn if stash items are accessed multiple times before promotion.
- Caveat: Stash hits still promote immediately, so the LRU policy distinction only matters if we also adopt H02's selective promotion.

Now I'm realizing the real distinction between ClassicLRU and Swap is in how they handle `get_from_position`: ClassicLRU moves accessed items to the back (full LRU), while Swap moves them one position closer to MRU. The eviction candidate is always index 0 in both cases. So if I use Swap in the target or probing tiers, hits would shift items incrementally rather than jumping them to MRU, which could matter when a bucket has many items and we want to avoid thrashing.

This creates a hybrid recency-frequency effect within the bucket—items that get hit repeatedly accumulate position advances and eventually become the hottest candidate, while rarely-hit items stay near the front. The trade-off is that with a small bucket size like 4, Swap would need 3 hits to move an item from position 0 to position 3 (MRU), whereas ClassicLRU does it in one hit. For a stable hot set workload, Swap would still eventually promote frequently-accessed items, but the question is whether the gradual ascent helps or hurts overall hit-rate.

I'm realizing that implementing per-tier policy differentiation would require changes to the bucket creation logic, but `factory.rs` is read-only and controls how buckets are instantiated across the segment. However, `dash_bucket.rs` is writable, so I could potentially create a different bucket variant with a different policy—I just need to figure out how the factory uses geometry to determine which buckets to create.

Since I can't modify the factory's contract, I need to work within the existing constructor calls. The `eviction_policy.rs` is writable and the `EvictionPolicyKind` is a settings parameter that the Java caller can select via JNI, though the harness probably sweeps different policies and ClassicLRU is currently the best performer.

Rather than changing the eviction policy globally, I should focus on what's actually modifiable in `dash_segment.rs`. I thought about applying LRU refresh to probing-bucket hits through the `promotion_get` path, but I already confirmed that `get_from_position` applies ClassicLRU's move-to-back, so probing hits already get LRU reordering.

Looking at the probing logic more carefully, both the probing-bucket path and the target-bucket path correctly invoke `get_from_position`, which means items are being moved to MRU in both cases.

The key difference with `cycle/0004/v002` is that it uses `InsertPosition::Back` instead of `Front` when inserting into target buckets. This means items arriving from stash land at the MRU position immediately, making them the hottest in their bucket right away—so just one more access promotes them to probing. This is a much more aggressive promotion strategy compared to the baseline.

For H02, I'm considering two-level probing where items in the probing bucket can be further promoted to a "super-hot" slot if they're accessed while being the hottest. But that's getting complicated. Let me step back and focus on what's actually implementable from the open questions: intra-tier eviction ordering, proportional re-entry positioning, and ghost-set tuning. For proportional re-entry, I'd need to insert items at calculated positions within the vector rather than just front or back, which means I'd need to extend beyond the current `InsertPosition` enum. The issue is that bypassing `put_at` with a raw insert might skip eviction logic, so I need to work within the existing promotion mechanism. Looking at how the current approach inserts at `InsertPosition::Back` for MRU re-entry, I could instead try inserting at the middle of the bucket as a compromise—aggressive enough to keep frequently accessed items hot, but not as extreme as making them immediately the hottest.

The cleanest way forward is probably using the `lfu_counter` to decide dynamically between inserting at the front or back, letting the access frequency guide where the item lands in the bucket. Looking at the actual overlay files, cycle/0004/v002 doesn't reset the `lfu_counter` in `promote_to_target`, so the counter value persists from the stash phase. Since ClassicLRU is the eviction policy being used, the `lfu_counter` value doesn't actually affect eviction decisions anyway.

Now I'm thinking through what genuinely novel approaches could work here. H02 could be something completely different from H01's ghost set modification—maybe focusing on scan resistance within the target tier, which the cross-cycle insights specifically call out as unexplored territory.

The challenge is that scan pollution was supposedly handled by the LFU-gated promotion in v001/cycle4, but it failed. Another angle is intra-probing ordering—using a different eviction policy just for the probing tier instead of applying one policy uniformly. The architecture constraint is that `EvictionPolicy` lives in `DashBucket`, and I can't modify `factory.rs`, but I could potentially override the internal policy behavior within `dash_bucket.rs` itself to differentiate between stash and normal buckets.

Wait, but `DashBucket` doesn't actually know whether it's a stash or normal bucket. Let me pivot to a completely different lever: **segment count**. The geometry allows tuning `num_of_segments`, which currently passes through unchanged. More segments means more independent hash tables and less collision pressure within each one. This could help if the workload clusters keys into the same segment. Since each segment maintains its own stash, target, and probing buckets independently, increasing segments creates finer-grained isolation—an unexplored topology axis. I'm considering H02 as a variant that increases segment count while keeping the 75/25 normal-to-probing ratio.

Actually, the current v002 geometry only adjusts `num_of_normal_buckets` and `num_of_stash_buckets` within a segment, leaving `num_of_segments` fixed. If I doubled the segments while halving the bucket count per segment, total capacity stays constant but each segment handles fewer items, reducing collision pressure. The total capacity formula is segments × normal_buckets × bucket_size + segments × stash_buckets × bucket_size, so doubling segments means halving both bucket counts while ensuring `num_of_normal_buckets` stays at least 2. For instance, with 1 segment having 24 normal + 8 stash buckets at bucket_size 4, I could shift to 2 segments with 12 normal + 4 stash buckets each, keeping total capacity at 32 buckets × 4.

But I'm realizing I don't have clear visibility into how `num_of_segments` versus `num_of_normal_buckets` actually maps to total capacity without more concrete data, so I should focus on ideas that are more grounded in what's already been tested. Looking at the geometry code, the harness provides the settings and `BaselineGeometry` just passes them through for production, while the evolution only tweaks the normal/stash split within a segment. Let me explore something more concrete instead, like increasing the bucket size itself.

Actually, `bucket_size` is a single global value applied to both normal and stash buckets, so changing it would affect total capacity across the board. I could look at what `item.rs` offers—items track `lfu_counter` and `timestamp`, and I could theoretically add a field like "tier_entry_count" to track how many tiers an item has crossed, but that feels overly complex without clear benefit.

Let me step back and design three concrete, testable hypotheses that are distinct and have real potential to improve hit-rate. The first one involves changing where ghost re-admitted items land in the queue—instead of placing them at the LRU end, I could insert them at the MRU end to accelerate their movement toward probing. target)
Change: ghost re-admitted items land at `InsertPosition::Back` (MRU end of target)
Why: The cross-cycle insight explicitly identified Front insertion as "neutralising most of the latency savings the bypass was meant to deliver." Re-admitted items at MRU would immediately be the promotion candidate in target, meaning one more access promotes them to probing. This directly tests the identified root cause.
Parent: cycle/0004/v002
Files: src/dash/segments/dash_segment.rs only

### Hypothesis 2 (parent: cycle/0003/v002):
**Larger ghost set (128 entries) + Back insertion for ghost re-admission**

Starting from cycle/0003/v002 (the cycle 3 winner with 24/8 split and MRU re-entry), I'm testing whether a larger ghost set at 128 entries combined with Back insertion for re-admission produces better results than H01. This differs from H01 because it builds the ghost set from scratch on a parent that doesn't have one yet, testing both the larger capacity and MRU insertion together rather than just changing insertion position. Since cycle/0003/v002 already uses Back insertion for stash→target promotion, keeping re-admission at Back maintains consistency.

Parent: cycle/0003/v002
Files: src/dash/segments/dash_segment.rs (add ghost set with 128 entries, Back insertion)

### Hypothesis 3 (parent: baseline or another prior variant):
I'm considering a two-probe target lookup approach where we check if the probing bucket has space before inserting into target, but that fundamentally changes the architecture too much. Instead, I'm exploring a different topology experiment within the 75/25 geometry: using wider stash buckets with fewer total buckets. The current setup uses 8 stash buckets with bucket_size of 4, giving 32 total stash slots. I'd test reducing to 4 stash buckets while adjusting the bucket size accordingly to see if wider buckets improve performance.

Actually, I'm realizing that bucket_size is a global parameter controlled by settings, not by geometry configuration. The geometry only controls the number of normal buckets, stash buckets, and segments. Since we've already established that the 24/8 split is optimal—with 26/6 and 28/4 showing regression—and the gain curve peaked at 24/8, exploring 22/10 (slightly more stash) likely won't help either direction.

But wait, I should double-check the cycle history: cycle 1 had 16/16 as the winner, then cycle 2 found 24/8 was better. Cycle 3 tested 28/4 and it regressed. We haven't actually tested 26/6 yet, even though the analysis suggests the topology lever is exhausted.

I think I need to shift my approach for the next hypotheses. Let me explore something different with H02: what if items in the probing tier get a second chance before eviction, where we check if their true target matches their current slot?

Looking at the existing code, foreign items already get demoted back to their true target with MRU positioning when evicted from probing, so they're already getting that soft second chance. I need to think about what else could improve the topology here. Defining the demotion logic...

Actually, I'm wondering if I should give every evicted item a second chance in its true target bucket before sending it to stash, not just the foreign ones. Right now native items that get evicted go straight to stash, but applying the same one-more-chance logic to all evicted probing items regardless of origin might be more consistent.

Let me trace through the logic more carefully—when an item is evicted from a probing slot, I need to understand whether that probing slot is actually the item's true target or not. The probing index is calculated as one position ahead of the target, so they're never the same unless the segment size is 1, which isn't valid. in bucket (B-1) as a probing slot for bucket B-1, then bucket B would be that item's true target. So an item could have its true target equal to the probing bucket index if its hash maps to a bucket that happens to be someone else's probing slot. When such an item gets evicted from its probing bucket, it goes straight to stash since there's nowhere else to place it. target bucket 6 was placed in bucket 6 as a "foreign" probing item (its original target was 5, probing at 6... no wait).

The probing placement is: if an item's true_target is 5, its probing slot is 6. If it gets promoted to probing, it goes to bucket 6. When bucket 6 evicts an item, and the evicted item has true_target == 5 (≠ 6), it gets one-more-chance in bucket 5. If the evicted item has true_target == 6 (an item that was there as its own target, not as someone else's probing), then true_target == source_idx == 6, and it goes to stash.

But wait: if an item's true target is 6, it would be IN bucket 6 as its target, not probing. If it then gets promoted, it goes to bucket 7. If it gets demoted FROM bucket 7, `source_idx = 7`, and `true_target = 6 ≠ 7`, so it gets one-more-chance in bucket 6.

So the `true_target == source_idx` case happens when an item is evicted from its own TARGET bucket. When an item is evicted from bucket X as a target-tier item and its true target IS bucket X, then `true_target == source_idx` → goes to stash. The `demote_from_normal_bucket` function handles both items evicted from probing slots and items evicted from target buckets, with the key distinction being whether the evicted item was in its own target bucket or someone else's probing slot.

If we changed the logic to send items with `true_target == source_idx` back to their target bucket instead of stash, we'd risk creating cycles where items keep getting evicted and re-inserted into the same bucket. The recursion concern is real: if a cascade item gets inserted into its target and displaces another item with the same target, we could have infinite recursion unless the stash eventually absorbs something. The recursion problem is that if I retry items in their target bucket, cascade evictions from that target would also get retried, creating an infinite loop. I could add a source tier parameter to break the cycle, but that adds complexity. Instead, let me try a simpler approach: just increase the ghost set capacity from 32 to 64 while keeping the current insertion strategy, which tests whether a larger ghost set alone improves performance. I'm realizing H01 and H02 would be testing essentially the same thing—both are about changing insertion position in the ghost set—just from different parent versions, which creates unnecessary duplication. Instead, H02 should explore a genuinely different mechanism: when a stashed item gets accessed and promoted back to the target, I'd preserve its MRU status by using Back insertion for the evicted item going into stash, rather than always promoting the oldest stash entry on collision. , when target is full and a stash item is promoted:
1. Stash item goes to target at `Back` (MRU) in cycle/0004/v002
2. Target evicts its LRU item → that item goes to stash at `Back` (MRU) in cycle/0004/v002

So target evictions that land in stash use `InsertPosition::Back`. This means demoted-to-stash items land at MRU of stash, and when stash is full, the stash's LRU item gets evicted. The stash LRU is the item that entered stash earliest and hasn't been accessed.

This seems reasonable. What about the position of the stash-promoted item when it enters target? Using `InsertPosition::Back` makes it immediately the hottest in target. This is what cycle/0002/v002 and all subsequent cycles do.

Now looking at H02, which is a fresh idea branching from cycle/0003/v002 about explicitly refreshing position on probing hits. In cycle/0004/v002, when we get a probing hit, we call the method that moves the item to the back via the underlying LRU mechanism—this already happens correctly. Let me check cycle/0003/v002's implementation since cycle/0004/v002 builds on top of it with the ghost set added. The base behavior without the ghost set should be inherited from cycle/0003/v002, and the promotion logic there uses the same approach.

Let me explore a genuinely different direction instead. What if, when promoting from stash, we check whether the item's probing bucket has available capacity? If it does, we could skip the target tier entirely and promote directly from stash to probing. This would reduce contention in target buckets and let more diverse items warm up there while the hottest items go straight to probing. The testable hypothesis is whether this direct path improves hit rates by easing target pressure.

Implementing this in the promotion logic...

Actually, this still preserves stash as a quality gate since items must hit stash first to be promoted—we're just changing the destination when probing has room. The concern is whether stash hits might pollute probing too aggressively, but since items only reach stash through prior access, they've already been filtered. The real question is whether this changes where they end up rather than whether they should be promoted at all.

For the FIFO approach, I'm realizing that stash already behaves like FIFO for non-accessed items since they never get repositioned. Explicitly switching to FIFO would only affect the eviction order of items that never hit, which might not meaningfully change the overall behavior.

A better direction would be batch eviction: when the stash overflows, evict both the LRU and second-LRU items at once. This aggressively trims the cold tail on each insertion, keeping only the warmest items in stash and potentially allowing items with genuine second hits to stay longer.

But this risks prematurely evicting items before they get a second access, which could actually hurt hit-rate.

Let me focus on three concrete, clearly distinct hypotheses instead. H01 changes ghost re-admission to use Back insertion instead of Front, directly fixing the bypass neutralization problem. H02 builds the ghost set with Back insertion and larger capacity. distinct, I should pivot H02 to target a completely different mechanism instead of just tweaking ghost set sizes. The idea is to adapt where stash-promoted items get inserted into the target bucket based on whether the target is already full and has hot items at the back—if so, insert at the front (LRU) to avoid displacing the hottest item and triggering a cascade. But that's complex to implement correctly, so I'm simplifying the approach for H02 to focus on a more concrete stash-promotion strategy.

Actually, I'm reconsidering whether changing the stash eviction policy helps at all, since stash hits always promote immediately anyway—the policy would only affect non-hit items, which behave the same under Swap, FIFO, or LRU. That doesn't seem like a real win. Instead, I'm thinking about using timestamp-aware eviction in the target bucket itself, where we evict the item with the oldest absolute timestamp rather than pure LRU recency, though I need to verify whether `TimestampLRU` and `ClassicLRU` are actually different in practice. The key distinction is that ClassicLRU's promotion candidate depends on access order (last position moved to back), while TimestampLRU's depends on actual timestamps—so if an item gets accessed and its timestamp updates, TimestampLRU might promote a different item than ClassicLRU would, even if both were moved to the same position.

The cross-cycle results consistently favor ClassicLRU over position-independent approaches, which suggests this difference matters. Now I'm settling on three concrete hypotheses to test: one explores ghost re-admission at the MRU end to make re-admitted items immediately eligible for promotion, another examines ghost re-admission from a different angle, and I need to define the third one.

Actually, I'm realizing H02 shouldn't duplicate H01's ghost set approach—it should test something genuinely different. Let me pivot H02 to explore an LRU-based alternative instead.

The key insight is how `lfu_counter` behaves during promotion from stash to target. In the baseline, the counter carries through unchanged, but the v001 overlay reset it to 0. Since v002 (the parent of v004) doesn't reset it, items retain their counter value when promoted. This means an item with `lfu_counter = 0` in stash stays at 0 when moving to target, which affects how it competes in the target tier's eviction logic.

Now I'm reconsidering the approach entirely. The H02 strategy shifts focus to stash insertion policy instead—when the stash fills up, evict the most recently used resident item (LIFO-style), treating cold items as higher priority than warm stash residents.

With ClassicLRU in the stash, items that never get accessed stay in FIFO order since there's no reordering happening, so the oldest-entered item gets evicted. But LIFO would flip this—the newest-entered item gets evicted instead, which could help if recently arrived cold items are less likely to be re-accessed than older stash items that have already survived without being touched.

However, for a stable workload where items outside the hot set just don't get accessed again, LIFO doesn't really help—an item is equally cold whether it's old or new.

The better approach is a counter-gated stash graduation: items need multiple stash hits before graduating to the target's MRU position, otherwise they stay in the stash. This acts as a multi-hit filter that prevents single-hit items from polluting the main cache.

Implementing this...

Implementing the counter logic in `promotion_get`... Writing the promotion logic...

Actually, I'm realizing this two-hit requirement is just another form of promotion gating, and the history shows that pattern consistently hurts performance on this workload. Items in the stash that are part of the active set will get accessed again anyway, so delaying their promotion to the hot tier only increases misses. This approach mirrors the failures from earlier cycles, so it's unlikely to improve things, but it's worth proposing as a distinct hypothesis even if it doesn't pan out.

For my final decision, I'm going with two hypotheses: one that directly addresses the root cause from cycle 4 by moving ghost re-admission from Front to Back insertion, and another that tests whether ghost set capacity itself is the bottleneck by using a much larger ghost set with Back insertion from a clean parent. Both would involve similar code changes, but they're testing different mechanisms—one targets the insertion position issue, the other tests whether we just need more ghost coverage. 32 vs 128) and starting points (v004/v002 with Front changed, vs v003/v002 with new ghost set). The output code is very similar. I should make H02 more distinct.

**H02 (cycle/0004/v002) — COMPLETELY different mechanism**:
"Demote items from probing to stash's MRU end (Back) instead of per-insert_pos"

Currently, when an item is demoted from target to stash, it uses `insert_pos` = `InsertPosition::Back`. But when it eventually arrives in stash, it competes with other stash items for LRU eviction. Since it's inserted at Back (MRU of stash), it's protected from immediate stash eviction. This is correct.

But what about the demotion cascade's stash insert? When a probing eviction causes a cascade to target, and THAT evicts a stash item... stash doesn't cascade further, so those items are just dropped entirely. In the ghost set variant, I track these evictions to help with re-admission, but that's a separate mechanism.

**H02 final (parent: cycle/0003/v002)**: Instead of promoting items to probing at the MRU end, I'll insert them at the LRU end to reduce the probing tier's eagerness to evict. This makes the tier less aggressive about keeping newly-promoted items around.

The change is to modify the `promote_to_probing` call to use the LRU insertion position instead of MRU. This way, newly-promoted items from target start at the LRU end of probing and must survive multiple eviction rounds before reaching MRU, making probing a slower-to-enter but longer-staying tier.

The testable hypothesis: Does inserting promoted items at the LRU end of probing reduce unnecessary eviction of established probing residents?

I'm realizing with ClassicLRU, the front/back semantics might be inverted...

I'm reconsidering the insertion position semantics and how they interact with the LRU ordering. I'm overcomplicating this. Let me just commit to three concrete experiments: H01 tests changing ghost re-admission from Front to Back insertion, H02 tests a larger 128-entry ghost set with Back insertion to see if capacity was the limiting factor, and I need to stop second-guessing and actually run these. similar. Let me try:

**H02 (parent: cycle/0004/v002)** — Adaptive stash insertion position based on bucket fullness
- When stash is full and evicts an item, that item goes to stash at `InsertPosition::Back` (current v002 behavior) — BUT the incoming item's position in stash is changed based on how "hot" the stash already is.
- Specifically: if ALL stash buckets are at >= 50% capacity, insert the new item at `InsertPosition::Front` (LRU end, evicted sooner) instead of `InsertPosition::Back`

No, this is confusing. Let me pick a completely different axis.

**H02 (parent: baseline or cycle/0002/v002)**: "Dual-pass eviction in stash: track items evicted from stash since last sweep, and on new arrivals, skip re-insertion into stash entirely if the item's hash bucket is already oversaturated"

This is too complex and vague.

**Let me just commit to a simple but distinct H02:**

**H02 (parent: cycle/0004/v002)**: Extend ghost set to also track items evicted from the TARGET tier (not just stash), giving recently-hot-tier-evicted items a fast path back to target on re-arrival.

Currently the ghost set only tracks keys evicted from stash. I want to add a second ghost set that tracks keys evicted from the target tier instead. When a target-evicted key re-arrives, it gets re-inserted directly into probing, skipping both stash and target since it was recently hot enough to belong there. This creates a genuine new mechanism: a "target-ghost" set that fast-tracks recently-target-evicted items back to probing. The implementation adds a second ghost set...

Adding ghost set tracking... The target-ghost path only triggers when an item has been evicted twice—first from target to stash, then from stash entirely—and then gets re-requested later. On re-arrival, these items can either go directly to target or into probing, bypassing the stash quality gate since they've already proven themselves through their time in target. evicted items, but for items that had a longer journey (target → stash → evicted). These items were "warm" recently (they were in target) and are now re-arriving. Giving them fast-track to target directly seems reasonable.

But wait - the current stash ghost set already tracks stash-evicted items. A target-evicted item first becomes stash-evicted (since target → stash → stash-evict). So the CURRENT ghost set in v002/c4 would already track these items (since they were evicted from stash, the last place they were before exiting the cache).

The difference: the current ghost set tracks "was in stash when evicted from cache." A target-ghost would track "was in target before stashing." This distinction might matter if the additional history (was in target) gives a better signal for where to re-admit.

For a target-ghost tracked item arriving via put(), I know it was recently in TARGET, so on re-arrival I can skip stash and go directly to target at the back. I'd need to track target-to-stash demotions separately by adding a target ghost set to the segment, then when an item gets demoted from target to stash, I record its key so I can recognize it on re-arrival and fast-track it back. ating information across functions. Let me step back and define three concrete hypotheses to test: one that directly addresses the root cause by re-admitting ghosts at the MRU position, another that adds a dedicated ghost set with back insertion from a clean slate, and a third that avoids displacing promotion candidates by inserting stash items at the LRU end when the target bucket is about to promote. Looking at the actual code flow, when a stash hit occurs in `promotion_get`, the item gets removed from stash and promoted to target via `promote_to_target`, which then calls `core.put_at` to insert it.

The key issue I'm seeing: inserting at the MRU position makes the newly promoted item immediately the hottest in its bucket, which means it becomes the next promotion candidate. This displaces whatever warm item was previously at MRU, potentially disrupting a stable set of items that were already performing well in that bucket. If the target bucket had established warm items, promoting the stash item to MRU could actually destabilize the working set rather than improve it.

That said, the empirical results show `InsertPosition::Back` is working well across the test cycles, which suggests this concern is more theoretical than practical. The workload is fundamentally recency-driven, so treating freshly-accessed items as hot makes sense.

I'm settling on three competing hypotheses to test: one focuses on fixing where ghost items get re-admitted into the target bucket, another tests using a larger ghost set capacity to capture more evicted items, and the third flips the insertion position for probing-tier promotions to the front instead, forcing new items to "earn" their position rather than starting as the hottest. The first two both involve ghost set improvements, while the third takes a different approach entirely.

For H02, instead of tweaking the ghost set, I should test whether items in the target bucket should stay in place when they're hit but aren't already the most recent. Right now, a hit always moves the item to the back (making it the hottest immediately), but what if we skip that repositioning and treat it like FIFO—hits don't change position, the back item is the promotion candidate, and the front item gets evicted first. Items promoted from stash would still land at the back and become immediately hot, but existing target items wouldn't get that same boost on every access.

The problem is this feels too conservative for items that have been sitting at the front of the bucket—they'd need everything above them to leave before they could ever become the hottest. That's the opposite of what I want. The real insight from the earlier cycle is that moving items to the back on hit is actually good, so FIFO within target would lose that benefit.

Let me try a different angle for H02.

With the `Swap` policy, accessed items shift one position toward the back per hit instead of jumping all the way there like ClassicLRU does. The eviction and promotion candidate indices stay the same (0 and len-1), but the key difference is in how `get_from_position` works—it moves an item up by one slot rather than to the end. This means an item needs multiple hits to climb from its entry position to the hottest spot, and only when it reaches that hottest position and gets hit again does it promote to probing.

For a bucket of size 4, an item inserted at the back is already hottest and needs just one more hit to promote, while an item inserted at the front needs three additional hits to reach that hottest position. This creates a gradual promotion ladder where items earn their way up through repeated accesses rather than immediately becoming hottest on a single hit. With ClassicLRU, any single hit moves an item to position 3 regardless of where it started, but with Swap, an item at position 0 only moves to position 1 on its first hit and needs two more hits to reach the top.

For workloads with a stable hot set... ), items are accessed frequently. Swap might help because:
1. Items that are somewhat hot (accessed 2-3 times) won't get promoted to probing INSTANTLY; they need a few more hits
2. This prevents "false promotions" where a recently-promoted stash item (at Back = hottest immediately) gets promoted to probing on one access, possibly displacing an established probing resident

For workload-a's stable hot set, the hottest items will definitely earn their way to probing through Swap's gradual mechanism. The key question is: does the gradual mechanism help or hurt the hot set's hit-rate?

This is genuinely untested. ClassicLRU is the baseline that has always won, but Swap is the only position-based alternative with the same eviction/promotion candidates (index 0 for eviction, len-1 for promotion) — meaning the fundamental structure is preserved, only the path to reaching MRU is different.

Now I'm looking at how demotion cascades work. When items get demoted from the target bucket to stash, the insertion position matters—the raw baseline uses Front-insert which sends demoted items straight to stash LRU and gets them evicted immediately under pressure, while the winning variants switched to Back-insert to place them at stash MRU instead. This is a key structural difference between the baseline and the cycle winners.

I'm also considering whether TimestampLRU could work better as an eviction policy. With TimestampLRU, the timestamp gets set when an item is created, and the eviction candidate is always the oldest timestamp rather than the oldest position. This means recently-accessed items with high timestamps stay protected, and the promotion candidate is the newest timestamp—which might not be at the vector's back position. I need to trace through when `Item::new()` is actually called to understand how timestamps flow through the system.

When items are cloned during promotion between tiers, they keep their original timestamp from initial cache insertion, not the access time. So for TimestampLRU, the promotion candidate becomes the most recently inserted item overall, not the most recently accessed one, and eviction targets the item that's been in the cache longest. This is fundamentally different from tracking access recency.

For a stable hot set, this creates a problem: long-lived hot items get old timestamps and become eviction targets, while newly arrived cold items get protected. TimestampLRU would perform worse than ClassicLRU here, so this isn't a viable direction.

The real hypothesis worth testing is whether front-inserting items during stash demotion prevents hot items that were displaced from the main tier from getting immediately re-evicted from the stash. But looking at the actual implementation in cycle/0003/v002, all operations including stash demotions use back-insertion, so...

Actually, I need to step back and clarify my hypotheses. Let me map out three distinct experiments: ghost re-admission at the back of the queue, swapping out the eviction policy entirely for something like Swap, and a baseline variant with a larger ghost set built from the raw baseline. The third one would need some additional setup work though. I'm settling on three distinct hypotheses instead. H01 targets the ghost set re-admission logic by switching from Front to Back insertion. H02 swaps out the eviction policy entirely—replacing ClassicLRU with Swap—though I need to check whether the harness allows policy overrides or if it's baked into the factory creation.

Looking deeper, the harness passes in the eviction policy as a boxed trait object, so overriding it in `dash_bucket.rs` would mean ignoring what the harness explicitly configured. That would break the harness's policy sweep experiments and defeat the purpose of testing different policies.

I could create a new policy variant that behaves like Swap, but the factory that creates `DashBucket` is contract-protected, so I can't change which policy gets passed in. Though `dash_bucket.rs` itself is writable, so maybe I can override the policy selection there...

Actually, I'm realizing I need to think through the Swap semantics more carefully—it moves items toward the back (MRU end), not forward, so I need to make sure I'm implementing the direction correctly. order becomes [1, 3, 2]. So Swap is moving frequently accessed items toward the front, which is the eviction point—that's the opposite of what I'd expect for an LRU cache. Let me trace through the logic more carefully to understand if this is intentional or if there's a bug in how the eviction candidate is set. I'm realizing the Swap policy might be backwards—it evicts from position 0 (front) while moving frequently-accessed items toward that same front position, which would mean it's actually evicting the most-used items instead of the least-used ones. Let me reconsider what "eviction candidate" and "promotion candidate" actually mean in this context, because this behavior seems counterintuitive for an LRU cache. stays at position 0). When I access key 2 at position 1, it swaps with key 1 at position 0, moving key 2 to the front and pushing key 1 back—so frequently accessed items actually move toward the eviction position. This is backwards from what I'd expect; the Swap policy seems to implement an anti-LFU behavior where accessed items become eviction candidates rather than protected ones.

But wait, let me reconsider the eviction logic. The `eviction_candidate_index` returns 0, confirming position 0 is the eviction point. So if Swap moves items toward position 0 with each access, it's creating a move-to-front algorithm—but one that evicts from the front instead of protecting from it. That would make this policy counterproductive for caching unless I'm misunderstanding the intent. So the swap function moves an accessed item one position toward the front by swapping it with the item at the lower index, and returns the new position. This means frequently accessed items gradually bubble toward position 0, where they're protected from eviction—that actually makes sense for a cache.

But wait, I'm seeing that after accessing key 3 twice, it ends up at position 0, which is marked as the eviction candidate. That would mean the most recently accessed item gets evicted, which is backwards. Let me check if the position indexing convention is inverted in this codebase, where position 0 represents the most recently used rather than the least recently used.

Looking at the ClassicLRU implementation, I can see it removes an item from its current position and pushes it to the back of the list, returning the new position at the end. The eviction candidate is always at position 0, and the promotion candidate is at the last position. So the convention here is that position 0 is the LRU end and the back of the list is the MRU end, which means accessed items get moved toward the back.

But Swap does the opposite—it moves accessed items toward position 0, which would cluster frequently-accessed items at the LRU end while infrequently-accessed items stay near the MRU end. That's completely inverted from what you'd expect. The tests only verify that position 0 gets evicted, which matches ClassicLRU's behavior, but they don't actually check whether Swap produces sensible caching results. It looks like Swap might be an intentional experiment that reverses the typical move-to-back strategy, though the tests don't validate whether that actually works.

Alright, I'm going to stop analyzing and just commit to three concrete, distinct hypotheses. H01 fixes the ghost re-admission by changing it to insert at the back instead of the front—that's a one-line change that directly addresses the root cause I identified. H02 adds a larger ghost set with 128 entries and uses back insertion. insertion from a clean parent that doesn't have the Front-insertion ghost set history
- Parent: cycle/0003/v002 (hit-rate 0.715, 24/8 geometry + Back insert, no ghost set)
- Changes: Add 128-entry FIFO ghost set in `DashSegment`; on ghost hit in `put()`, call `promote_to_target(..., InsertPosition::Back)`
- Tests whether ghost set size AND insertion position together produce material gain when done right from the start

Yes, H01 and H02 are related (both about ghost set + Back insertion) but differ in parent and ghost set size. H01 is minimal (change one line in v002/c4), H02 is a fresh build with larger set. They'll produce similar but not identical code.

Actually, H01 and H02 would produce nearly identical code at the end:
- H01: v002/c4 ghost set (32 entries) with Back insertion (one line changed)
- H02: v003/c3 + new ghost set (128 entries) with Back insertion (new code)

The key difference is ghost set SIZE (32 vs. 128). This IS worth testing as it answers "was the ghost coverage the problem or was it the insertion position?"

For H03, I'm testing a different angle: when an item already in the target gets hit but isn't the hottest, skip the reordering step entirely instead of moving it to the back like ClassicLRU does. Only the hottest target hits that trigger probing promotion get special treatment, leaving everything else in place. to probing would differ significantly: items enter target at the back when promoted from stash and become immediately hottest, then get promoted to probing on their next hit. But if target fills up and displaces that item, subsequent hits wouldn't move it back to the hottest position—it would just stay where it is until other items above it eventually leave. This is fundamentally different from ClassicLRU's behavior of always moving accessed items to the back, and I'm uncertain whether this change actually improves performance or if items benefit more from being repositioned to hottest when accessed again after displacement.

For workload-a's stable hot set where items are accessed frequently, ClassicLRU's move-to-back strategy keeps the most recently accessed item at the back, ready for the next promotion to probing. Without this move, items that have been sitting in target for a while would stagnate in lower positions even if they're being accessed regularly, which seems likely to hurt hit-rate. The reason ClassicLRU keeps winning is probably because it ensures frequently-accessed items stay in the hottest position.

I need to stop cycling through the same arguments and actually look at what the cross-cycle insights identified as unexplored territory.

Looking at the open questions: segmented LRU within the hot tier hasn't been explored yet, proportional re-entry position based on access count is untested, and ghost-set tuning is only partially explored. Let me think about what "segmented LRU within the hot tier" actually means—it's a two-tier system where items start in a probationary segment and move to a protected segment when accessed, with protected items that get evicted dropping back to probationary rather than leaving the cache entirely. I could map this onto our target and probing segments.

Actually, I'm realizing the Dash cache already implements this pattern—target acts as the probationary segment and probing as the protected segment. The scan resistance comes from requiring items to be accessed at least twice before reaching the protected tier, which prevents one-time scans from polluting the cache. To improve scan resistance further, I'd need to either add another tier or raise the access count threshold for promotion.

Looking back at the baseline, there was an attempt to require `lfu_counter >= 2` for target→probing promotion, but that actually regressed hit rates. The most promising unexplored approach is proportional re-entry positioning—when promoting a stash item to target, I'd insert it at a position based on its `lfu_counter` rather than always placing it at the front or back. Items with higher access counts would go to the MRU end while never-accessed items go to the LRU end. The tricky part is that stash items start with `lfu_counter = 0`, and they get incremented when hit during promotion.

Implementing this means I need to increment `lfu_counter` in the stash hit path before promoting to target, then use that counter to determine the insertion position...

Writing the promotion logic... I'm realizing the core issue: items promoted from stash on their first hit would always have lfu_counter == 1, making the insertion logic deterministic rather than frequency-based. What if instead I track the lfu_counter from when items were demoted into stash—items that had previous target access would carry that history, so re-accessing them in stash could use that prior frequency to decide insertion position. But this breaks down with ClassicLRU as the eviction policy, since it never increments lfu_counter in the target layer, leaving everything at zero.

The solution is to increment lfu_counter independently whenever an item is accessed in target, regardless of eviction policy—this way the counter captures how frequently an item was used before demotion, and when it returns from stash, that history informs whether it gets inserted at the back or front.

However, the experimental data shows lfu_counter hasn't helped in practice, so I'm moving forward with three concrete hypotheses instead, accepting that H02 and H03 might not outperform the baseline.

For H01, I'm fixing the root cause by changing ghost re-admission in DashSegment::put() from Front to Back insertion. For H02, I'm adding a 128-entry ghost set using a VecDeque that tracks evicted keys, checking it on put() to re-admit items at the back of target, and recording new evictions to the ghost set.

Now I'm reconsidering H03 — the LRU suppression approach where non-hottest target hits don't move to MRU, preserving insertion order except for stash promotions landing at Back. I initially thought this would hurt performance, but I'm second-guessing that conclusion based on the cross-cycle insights.

The key change is modifying how target hits are handled: instead of reordering the item to MRU on access, I'd return it directly without position changes, so only fresh stash→target insertions create new hottest candidates. This could help if frequently-accessed target items that weren't the most recent stash insertion are being evicted prematurely because they keep climbing the MRU ladder.

But this breaks the core LRU semantic where accessing an item should make it hotter and less likely to evict. Items cycling between stash and target would get stuck in their original insertion positions and evict sooner than they should. This approach would likely hurt performance.

Instead, I'm going with H03 FINAL: use pure FIFO insertion order in target, where items promote based on when they entered target, not on access recency. This is closer to the original baseline behavior before the cycle improvements.

Now I'm reconsidering my hypothesis options. Rather than settling for near-duplicate variations, let me look back at the unresolved questions from the cross-cycle analysis. The ghost-set tuning has two distinct levers I haven't fully explored: size and insertion position. H01 tests position (Front to Back), and H02 could test both size (32 to 128 entries) and position together, which would give me genuinely different experiments to run.

But the real opportunity is in scan resistance within the hot tier. The hot tier consists of the target and probing buckets, and I could implement segmented LRU behavior where the target acts as probationary and probing acts as protected. The key insight is preventing cold items from scans from displacing hot items—when a stash item gets promoted to target and would trigger an eviction, I'd check whether the eviction candidate is actually a hot item that should be protected, and if so, handle it differently.

The challenge is measuring "hotness" to make that comparison. Since `lfu_counter` stays at zero for ClassicLRU items, I need another signal. One approach: increment `lfu_counter` when an item gets hit while in stash, so the counter represents how many times it's been accessed from stash before promotion. This gives me a concrete way to distinguish between items that just entered stash versus those that have been repeatedly accessed there. So the key insight is using the counter as a history signal—items that have cycled through stash before (lfu_counter > 0) have proven reuse and should enter at the back of the target list as MRU, while first-time stash exits enter at the front and have to climb. I'd increment the counter on stash promotion and then check it during insertion to decide placement. I'm realizing the issue: I need to increment the counter during the stash hit but check it before incrementing, so new items from the stash get Front insertion while items that have been hit multiple times get Back insertion. The key is checking the counter's current value to decide placement, then incrementing it afterward so subsequent hits reflect the increased frequency. second stash entry. So the mechanism holds: items that cycle through stash multiple times accumulate higher lfu_counter values, which shifts them from conservative Front insertion toward aggressive Back insertion on re-promotion.

The real question is whether this distinction actually helps—the history shows Back insertion worked well for proven reuse, but I'm worried most hot-set items will quickly exceed lfu_counter > 0 and get Back treatment anyway, leaving only truly cold items (single access ever) getting Front insertion. That might be too narrow to matter.

H03 is genuinely different from H01 and H02 because it conditions insertion position on access history rather than just using Back uniformly for ghost re-admission. But I need to pick the right parent: using baseline is problematic since it lacks the 24/8 geometry and uses Front everywhere.

Cycle 2's v002 winner already has the 24/8 geometry with Back insertion, so layering the history-aware promotion mechanism on top of that would be cleaner than starting from baseline. Cycle 3's v002 is one generation newer and includes the MRU re-entry logic, but I should verify whether it already uses Back insertion or if that's something I'm adding fresh. Looking at the progression, cycle 2's v002 winner used just the 75/25 geometry with standard insertion, while cycle 3 compounded that topology with MRU back-insertion to get a measurable improvement. So for H03, I'd start from the cycle 2 v002 baseline and add a history-aware conditional that switches insertion strategy based on whether it's the first access.

Actually, using cycle 3 v002 as the parent makes more sense—it's the cleanest recent version, and the change becomes clearer: shifting from unconditional back-insertion to conditional front/back based on the lfu_counter. H02 also builds from cycle 3 v002, which is fine since they explore different modifications from the same foundation.

Now I'm considering whether H03 should instead branch from cycle 4 v002 and modify ghost re-admission logic, but I need to think through when ghost-tracked keys actually get re-inserted and what their lfu_counter state would be at that point.

When a key that was evicted and recorded in the ghost set gets requested again, it arrives as a fresh Item with lfu_counter at 0, so there's no historical counter information available unless I look it up separately—which adds complexity.

I'm settling on H03 as the history-aware stash promotion mechanism from cycle 3 v002, which only requires changes to the promotion logic in dash_segment.rs and doesn't need to track historical counters.

So my final hypotheses are: H01 changes ghost re-admission from Front to Back insertion, and H03 makes stash promotion conditional based on the lfu_counter value.

### 13:21:54 thinking

Let me now synthesize the three hypotheses based on everything I've read.

## Summary of what I know:

**Architecture**:
- `dash_segment.rs`: `put` → stash; `get` → promotion ladder (stash → target → probing)
- `geometry.rs`: controls normal/stash bucket split
- `eviction_policy.rs`: ClassicLRU (move-to-back on hit), also TimestampLRU, LIFO, LFU, FIFO, Swap
- `item.rs`: key, value, lfu_counter, timestamp
- `dash_bucket.rs`: concrete bucket implementation


- `cycle/0004/v002` is my best performer so far with a 71.47% hit rate, using a 75/25 geometry split and a 32-entry FIFO ghost set that re-admits items at the front of the target bucket

I'm now looking at what's worth testing next: the ghost set insertion position is currently placing re-admitted items at the front, but I could experiment with moving them to the back instead to see if that changes the dynamics.

I'm also considering whether the stash eviction policy could benefit from a gentler approach than the current LRU movement, and whether I should expand the ghost set capacity beyond 32 entries to catch more potential hits.

Let me formalize three concrete hypotheses to test. First, I'm going to try moving ghost re-admissions to the back position in the segment code to see if that improves hit-rate by making re-admitted items immediately the most recently used.

Modifying the put method... I need to figure out what the most informative hypotheses are to test. Option A would involve swapping the eviction policy within just the probing tier, which would require architectural changes to support per-tier policies—I'm not sure if `SegmentCore` already has that capability or if it would need modifications to `dash_segment.rs` and the eviction policy module.

Looking more closely at the code structure, each `DashBucket` has its own `eviction_policy` field, so theoretically I could create different policies for different tiers. However, the `SegmentCore::try_new` method that builds these buckets lives in `src/dash/segments/segment.rs`, which isn't in the extension-point files I can modify, so this approach is blocked.

Option B is more promising: when items get demoted through `demote_from_normal_bucket`, they're inserted at the same position as the triggering promotion. Since `cycle/0004/v002` uses `InsertPosition::Back`, demoted items land at the MRU end of the stash, which immediately makes them eligible for re-promotion—this could be creating a problematic cycle.

Actually, wait—I need to reconsider how stash promotion works. Items in stash get promoted to target when accessed, but their position in stash only affects eviction order, not promotion eligibility. With `InsertPosition::Back`, demoted items survive longer in the LRU order, and since they land at MRU, recently-demoted items get priority for re-promotion, which seems backwards.

Option C would flip this: demote to `Front` instead, forcing demoted items to prove themselves again before competing with naturally-arrived items. I'd modify the demotion logic to always use `InsertPosition::Front` while keeping promotions at `Back`, creating an asymmetry where demotions are treated as lower-priority entries.

Option D explores whether a larger ghost set (96 or 128 entries) could help—the idea being that keys cycling through stash eviction might not re-arrive within the current window, or the `Front` insertion position might be neutralizing the latency gains.

Now I'm narrowing down to three testable hypotheses: H01 tests whether re-admitting at `Back` instead of `Front` fixes the neutralization problem, H02 tests asymmetric demotion where demoted items start at `Front` of their destination tier while promotions still go to `Back`, and H03 explores a genuinely different approach like a new stash eviction policy or geometry change. I need to be careful about parent selection since at least one hypothesis must cite a cross-cycle winner. H01. The other two can be anything.

Let me think of something for H02 that's distinct from H01:

**H02: Asymmetric demotion insertion (cycle/0004/v002 parent)**
Change `demote_from_normal_bucket` in `dash_segment.rs` to always use `InsertPosition::Front` regardless of the `insert_pos` parameter. Currently, demoted items inherit the same `insert_pos` as the triggering promotion (which is `Back` in cycle/0004/v002). Making demotions land at LRU (Front) could reduce the "noise" in each tier — newly-demoted cold items won't immediately compete at the same priority as recently-accessed items.

**H03 parent: cycle/0003/v002 (which had the clean MRU re-entry + 75/25 topology without ghost set) OR baseline**

Looking back at the results, cycle/0003/v002 achieved 0.715 hit-rate with MRU re-entry and 75/25 topology, while cycle/0004/v002 added ghost re-admission but only matched that at 0.7147. For a genuinely different direction, I could explore a new eviction policy variant, though that would affect all buckets uniformly since the policy is applied globally. The `Swap` policy moves items toward MRU on each access like a clock-based approach, so for the stash I'd need to think through how that changes item positioning with repeated accesses. when hit? I could track this with `lfu_counter` — reset when an item enters the stash, then increment on each stash hit. The item structure already has this field and the stash preserves it across insertions, so it's available without modification. But this approach mirrors the LFU/threshold-gating attempted in cycle/0004/v001, which failed because the threshold of 2 was too conservative for this workload.

Now I'm considering whether the probing tier itself could function as a segmented LRU — the cross-cycle notes mention this as an open question for intra-tier eviction ordering. Currently probing hits use `get_from_position` with ClassicLRU, which moves items toward the MRU end.

What if I split the probing bucket into protected and probationary segments? Newly promoted items would enter at the boundary between them, while items that get re-accessed in probing move to the protected MRU end. Items that never get re-accessed would age out from the probationary side. I could implement this by inserting newly promoted items at the Front and letting subsequent hits move them to the Back, creating a natural segmentation without needing a new insertion position. Both promotion and hit operations place items at the MRU position, so the only distinction is tenure — older items drift toward LRU and get evicted first. If I switched promotion to land at the LRU position instead, newly-promoted items would need another hit to reach safety, creating a probationary filter similar to SLRU. But this approach has already been explored through the baseline and winner variants.

Let me pivot and explore what hasn't been tested yet — like increasing the ghost set size beyond the current 32 entries, or other structural changes to how the probing tier filters candidates. 128 entries better captures the working set that cycles through stash eviction? The ghost set is currently bounded by `GHOST_CAPACITY = 32`. The cycle/0004/v002 insight noted the workload "does not cycle heavily enough at the stash boundary." But this was with 32 entries. A larger ghost set could capture more stale-but-recurring keys.

This is a very simple change: just increase `GHOST_CAPACITY` from 32 to 128. Parent: cycle/0004/v002. But this is similar to H01 in that both target the ghost set. Too close?

2. **Stash eviction policy variant: Swap instead of ClassicLRU**
   This would require changing `eviction_policy.rs` to add a new policy that uses `Swap` for stash items but preserves position-based behavior for normal buckets. But the policy is applied uniformly across all bucket types. Unless I modify the `Swap` policy to work differently, or add a new policy like `SwapStash` where the behavior differs? Actually, this is a global policy change, not per-tier. If I switch to Swap globally, normal buckets also use Swap, which would probably hurt since items need multiple hits to reach promotion candidate position.

3. **Probing hit triggers re-insertion at MRU with a twist**: When an item is found in probing, rather than just returning it in-place, also check if this is the "hottest" probing item and if so, do something special like re-promoting it to reset its position. This is too complex and non-standard.

4. **Non-stash demotion path changes**: When items are demoted from probing to target, they currently land at `InsertPosition::Back`. What if they land at `Front` instead? This means demoted items start at the bottom of the target ladder and must re-earn their way up, testing whether the current MRU demotion behavior is hurting by displacing fresh stash-promoted items.

I think the most promising three hypotheses are: **H01** changes ghost re-admission to `Back` instead of `Front` to directly address the insight that Front-insertion neutralizes latency savings; **H02** makes demotion asymmetric by always using `InsertPosition::Front`, testing whether demoted items starting at LRU protects genuinely-hot items; **H03** explores proportional stash-to-target insertion position, which hasn't been tried yet and represents genuinely new territory.

For H03, when a stash hit promotes an item to target, I'd use the item's `lfu_counter` to bias insertion: items with prior accesses land at `Back` (MRU, fast-tracked), while first-time stash graduates land at `Front` (LRU, must earn position). This differentiates returning stash graduates from newcomers and tests whether that distinction improves hit-rate. The change would live in the promotion logic, checking the counter before deciding where to insert the promoted item.

But I need to verify the baseline behavior—checking v002's `promote_to_target` function, it doesn't reset `lfu_counter` like v001 does, so the counter persists through promotion and I can rely on it to distinguish first-time from repeat stash graduates. I'm considering a simpler hypothesis: expanding the ghost set capacity from 32 to 256 to see if the retention window for tracking evicted stash keys was the limiting factor. This is distinct from H01 since it tests coverage breadth rather than insertion positioning, and both questions are independent enough to warrant separate experiments. could be the real bottleneck. They are distinct strategies.

But I worry both H01 and H02 target the ghost set differently. That could be fine — one targets the ghost set's effective throughput (how quickly re-admitted items can act as hits), and the other targets its recall (how many evicted keys are remembered).

Actually, maybe for H03 I should go further afield and explore the asymmetric demotion idea (which is unrelated to the ghost set), using baseline or a prior loser as parent.

Let me decide:

1. **H01 (cycle/0004/v002)**: Ghost re-admission at `Back` — tests position, targets the `neutralization` issue from the insight
2. **H02 (cycle/0004/v002)**: Asymmetric demotion (`Front` for all demotions) — tests whether demoted items starting at LRU of their destination tier reduces displacement of hot residents
3. **H03 (cycle/0003/v002 parent)**: Test whether using `Front` insertion for stash promotions while keeping the 75/25 geometry and no ghost set creates cleaner separation — the hypothesis is that cycle/0004/v002's `Back` placement forces newly-graduated items to immediately compete as hottest in target, whereas `Front` would require them to earn MRU status through actual accesses, and cycle/0003/v002 already showed promise with this approach.

Actually, that H03 is a regression test since cycle/0003/v002 already demonstrated that `Back` is superior. Let me think more systematically about what's genuinely unexplored.

I've already tested topology (peak at 24/8), MRU re-entry with `Back` insertion (winning since cycle/0002), LFU eviction (always loses), ghost re-admission (marginal gains), LFU-gated promotion thresholds (loses), and stash bypass (destroys hit rate).

What hasn't been explored yet: ghost re-admission at `Back` instead of `Front`, and asymmetric demotion where demoted items land at LRU of their destination tier instead of the current approach.

I'm going to start with H01 — modifying the ghost re-admission logic to insert at `Back` rather than `Front` when promoting items back to the target tier.

For H02, I'm testing whether newly-demoted items should always start at the LRU end of their destination, so I'll force `Front` insertion in the demotion path regardless of the original insertion position.

H03 expands the baseline ghost set to 128 entries and re-admits at `Back` to isolate whether a larger ghost set alone improves performance.

But I'm realizing the parent branch matters—using `baseline` reverts the topology back to 8/24, which loses the 75/25 split gains. I should parent H03 off `cycle/0003/v002` instead, which already has the 75/25 topology and MRU re-entry, so adding the ghost set tests the compound effect directly.

Let me trace back through the ledger to confirm the lineage: `cycle/0001/v003` established the 50/50 split, `cycle/0002/v002` moved to 75/25, `cycle/0003/v002` added MRU re-entry on top of that, and now `cycle/0004/v002` is layering ghost set functionality with back insertion for promotions. Looking at the overlay files, `cycle/0004/v002` has both the geometry changes and the dash segment modifications, so that confirms the parent chain.

The issue is that H01 and H03 would both be testing back insertion for ghost re-admission, just from different starting points—that's too similar. Let me pivot H03 to something genuinely distinct: probing demotion behavior by inserting items at the front of the target list instead.

When an item gets evicted from probing and demoted back to its true target, it currently lands at the back (most recently used). The idea is to place it at the front (least recently used) instead, since it "failed" to stay hot enough in probing and should compete fairly rather than jump to the head of the queue. I'd modify the demotion logic to use front insertion specifically when moving from probing back to the true target, while keeping the existing behavior for target-to-stash moves. This creates a meaningful asymmetry distinct from H02's approach.

Now I'm settling on three concrete hypotheses to test. H01 focuses on changing ghost re-admission from front to back insertion to let re-admitted items immediately qualify for promotion. H02 is about making all demotion insertions use front (LRU) instead of inheriting back (MRU) from promotion context to prevent demoted items from getting unfair priority. to the target tier. But wait — implementing per-tier policies would require architectural changes I don't have access to with the current extension points. Let me refocus on the proportional stash insertion approach instead. I'm tracking hit counts and using them to determine insertion position: items with prior stash hits go to the back of the target tier to preserve their momentum, while first-time graduates enter at the front to make them compete for space. Modifying the promotion logic in the segment code to implement this... I need to explicitly increment `data.lfu_counter` in the stash hit path of `promotion_get` before deciding on insertion position, since ClassicLRU won't track those hits itself. The key question is whether the target version resets the counter or lets it accumulate — I should check what cycle/0003/v002 actually does with counter management. I'm thinking through the mechanics of using `lfu_counter` to decide insertion position during stash graduation—tracking how many times an item has cycled back from stash to target, then placing frequently-graduated items at the MRU end and cold items at the LRU end. The key question is whether to use the counter value before or after incrementing it to determine position, since that affects whether first-time stash graduates land cold or warm.

Looking at the baseline behavior, the original code always used Front insertion, but the big win came from switching to Back insertion in later versions. If I revert first promotions back to Front, I risk losing that improvement, so maybe the better approach is keeping Back for initial stash→target moves but adding a more aggressive bonus for items that graduate multiple times—though that's getting complicated and error-prone.

Let me reconsider the whole strategy from scratch.

For H03, I need something genuinely distinct from H01 and H02, grounded in an actual parent variant, and testable without being overly complex. Looking at what hasn't been explored yet: the Swap policy would likely hurt globally, changing the probing bucket's insertion position from Back to Front could create an interesting probationary zone for newly promoted items, a hybrid windowed LFU combining LRU with access-count minimums is risky given past LFU failures, adjusting bucket size or segment count could work but might conflict with existing constraints.

The geometry struct has several levers I haven't fully exploited—particularly `num_of_segments`. The harness sweeps this as part of the outer loop and the baseline geometry respects whatever the harness provides, but I could reinterpret these values through the geometry extension point. Looking at how BaselineGeometry calculates the layout, it distributes the total buckets between normal and stash buckets using a 3:1 ratio.

The question is whether increasing `num_of_segments` would help—more segments could improve hash distribution and reduce lock contention, plus smaller segments might have better cache locality. But the harness is already sweeping segment counts, and if I override `num_of_segments` in the geometry, I'd be fighting against what the harness is trying to test rather than working with it.

I'll stick with the probing insertion strategy instead. The idea is to insert newly-promoted items at the LRU end of the probing list rather than the MRU end, creating a probationary zone where they have to prove themselves with another access before they can compete with established hot items for MRU protection. I'll modify the promotion logic in `dash_segment.rs` to test whether this probationary approach improves hit rates. bing(core, data, insert_pos)` where `insert_pos = Back` to use `InsertPosition::Front`
- Keep stash→target at `Back` (the core MRU re-entry improvement)
- This creates asymmetry: stash→target gets MRU, but target→probing gets LRU of probing
- When a probing item is re-accessed, ClassicLRU's `get_from_position` moves it to Back (MRU)

So a newly-promoted probing item starts at Front. If it gets accessed in probing, it moves to Back (MRU) and is well-protected. If it doesn't get accessed in probing and gets evicted, it was a "one-shot promotion" — this filters stale promotions.

This is distinct from H01 (ghost set Back) and H02 (asymmetric demotion), which operate in different directions and tiers. These three hypotheses are genuinely unexplored territory around intra-tier eviction ordering.

Now I'm ready to formalize H01: inserting ghost-re-admitted items at the MRU end instead of the LRU end should improve hit-rate by letting them immediately become promotion candidates. The change is straightforward — modify the ghost re-admission logic in the segment code to use the MRU position, keeping all other ghost set behavior intact.

For H02: hard-coding demotion insertions to the LRU end rather than inheriting the promotion's MRU position should improve hit-rate by ensuring demoted cold items start at the bottom of their destination tier's eviction queue. The modification replaces the flexible insertion position with a fixed LRU position in the demotion function, though the function signature stays the same to avoid touching calling code.

For H03: promoting target items into probing at the LRU end instead of the MRU end should improve hit-rate by creating a probationary zone where newly-graduated items must earn an additional access before being protected from eviction. The change is to modify `promotion_get` in `dash_segment.rs` to insert promoted items at `Front` instead of `Back`, while keeping stash-to-target insertions at `Back` and all other logic unchanged. Let me verify that cycle/0003/v002 actually implements this and won the competition.

Looking at the cycles ledger, cycle/0003/v002 won with the "MRU re-entry compound" — it had the 75/25 topology from cycle/0002/v002 plus the MRU re-entry behavior in promotions. So cycle/0003/v002 is the baseline without ghost sets, and cycle/0004/v002 adds ghost sets on top. For H03's probationary probing test, I should build on cycle/0003/v002 to keep it minimal, changing only the promotion insertion position.

Now I'm ready to write the three hypothesis files for cycle 0005.

### 13:21:54 chat

Now I have all the context needed. Let me write the three hypotheses.

### 13:22:03 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/hypotheses/H01.md",
  "content": "# H01\n\nInserting ghost-re-admitted items at `Back` (MRU end) of their target bucket instead of `Front` will improve hit-rate by allowing re-admitted items to immediately qualify as promotion candidates rather than needing to earn MRU position first.\n\n## Parent\n\ncycle/0004/v002\n\n## Changes\n\nIn `src/dash/segments/dash_segment.rs`, inside the `put()` method of `DashSegment`, find the ghost re-admission branch that fires when a key is found in `self.ghost_keys`:\n\n```rust\nif let Some(pos) = self.ghost_keys.iter().position(|k| *k == item.key) {\n    self.ghost_keys.remove(pos);\n    promote_to_target(&mut self.core, item, InsertPosition::Front);  // ← change this\n    return;\n}\n```\n\nChange `InsertPosition::Front` to `InsertPosition::Back`:\n\n```rust\nif let Some(pos) = self.ghost_keys.iter().position(|k| *k == item.key) {\n    self.ghost_keys.remove(pos);\n    promote_to_target(&mut self.core, item, InsertPosition::Back);   // ← Back\n    return;\n}\n```\n\nAll other ghost set logic is unchanged: the 32-entry FIFO capacity (`GHOST_CAPACITY = 32`), the recording of stash-evicted keys into `ghost_keys`, and the ghost-key removal on re-admission. The normal cold insertion path (`core.put_at(Slot::Stash, InsertPosition::Back, item)`) is also unchanged.\n\nThe `promote_to_target` helper itself is unchanged; only the call-site in the ghost branch changes its `insert_pos` argument.\n\n## Rationale\n\nThe cycle/0004/v002 insight explicitly identified `Front`-insertion as the likely reason ghost re-admission yielded near-zero gain: \"the Front-insertion position chosen for re-admitted items (requiring them to earn MRU before promoting) neutralises most of the latency savings the bypass was meant to deliver.\" Using `Back` instead places re-admitted items at MRU of their target bucket immediately, making them the promotion candidate on the very next access — which is the full benefit the ghost bypass was designed to deliver.\n"
}
```

### 13:22:19 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/hypotheses/H02.md",
  "content": "# H02\n\nHard-coding demotion insertions to `Front` (LRU end) in `demote_from_normal_bucket` instead of inheriting `InsertPosition::Back` from the triggering promotion will improve hit-rate by ensuring freshly-demoted items start at the bottom of their destination tier's eviction queue rather than jumping to MRU and displacing established warm residents.\n\n## Parent\n\ncycle/0004/v002\n\n## Changes\n\nIn `src/dash/segments/dash_segment.rs`, modify the body of `demote_from_normal_bucket` to use `InsertPosition::Front` for every `core.put_at` call inside the function, ignoring the `insert_pos` parameter that is passed in. Do not change the function signature or any call sites — only the internal call-site argument changes.\n\nCurrent body:\n```rust\nfn demote_from_normal_bucket<K, V>(\n    core: &mut SegmentCore<K, V>,\n    item: Item<K, V>,\n    source_idx: usize,\n    insert_pos: InsertPosition,\n) where ... {\n    let true_target = get_index(&item.key, core.segment_size);\n    if true_target == source_idx {\n        core.put_at(Slot::Stash, insert_pos, item);          // ← change to Front\n    } else {\n        if let Some(cascade) = core.put_at(Slot::Target, insert_pos, item) {  // ← Front\n            core.put_at(Slot::Stash, insert_pos, cascade);   // ← Front\n        }\n    }\n}\n```\n\nReplace all three `insert_pos` arguments in `put_at` calls with `InsertPosition::Front`:\n\n```rust\nfn demote_from_normal_bucket<K, V>(\n    core: &mut SegmentCore<K, V>,\n    item: Item<K, V>,\n    source_idx: usize,\n    _insert_pos: InsertPosition,   // retained for signature compatibility, unused\n) where ... {\n    let true_target = get_index(&item.key, core.segment_size);\n    if true_target == source_idx {\n        core.put_at(Slot::Stash, InsertPosition::Front, item);\n    } else {\n        if let Some(cascade) = core.put_at(Slot::Target, InsertPosition::Front, item) {\n            core.put_at(Slot::Stash, InsertPosition::Front, cascade);\n        }\n    }\n}\n```\n\nAll promotion paths (`promote_to_target`, `promote_to_probing`) are unchanged and continue to insert at `InsertPosition::Back` (MRU). Only the demotion path diverges. The ghost set logic and geometry are also unchanged from the parent.\n\n## Rationale\n\nIn cycle/0004/v002 every `put_at` — both promotions and demotions — uses `InsertPosition::Back`. This means a demoted item from probing or target lands at MRU of its destination tier, competing immediately with the most recently-accessed residents. An item that just got displaced was less recently-accessed than the item that displaced it; placing it at LRU (Front) of its new tier gives it a fair position without letting it crowd out genuinely hot residents. This asymmetry — promotions at MRU, demotions at LRU — is standard in segmented-LRU designs and has not yet been tested in this cache.\n"
}
```

### 13:22:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/hypotheses/H03.md",
  "content": "# H03\n\nInserting newly-promoted items into their probing bucket at `Front` (LRU/probationary end) instead of `Back` (MRU end) will improve hit-rate by creating a probationary zone in probing where recently-graduated items must earn at least one additional access before being protected from eviction, filtering stale one-off promotions.\n\n## Parent\n\ncycle/0003/v002\n\n## Changes\n\nIn `src/dash/segments/dash_segment.rs`, locate the `promotion_get` function. Inside the branch that fires when an item is found in its target bucket as the hottest item (`is_hottest == true`), there is a call to `promote_to_probing`:\n\n```rust\nif is_hottest {\n    let data = core.buckets[target_idx].remove_at(item_idx);\n    promote_to_probing(core, data, insert_pos);   // ← change this argument\n    ...\n}\n```\n\nChange the `insert_pos` argument to hard-coded `InsertPosition::Front`:\n\n```rust\nif is_hottest {\n    let data = core.buckets[target_idx].remove_at(item_idx);\n    promote_to_probing(core, data, InsertPosition::Front);  // ← Front\n    ...\n}\n```\n\nNo other changes: stash→target promotion continues to use `InsertPosition::Back` (MRU re-entry, inherited from cycle/0003/v002), demotions continue to use the inherited `insert_pos`, and the geometry (75/25 normal/stash split) is unchanged. The `promotion_candidate_index` and `unreachable!` assertion inside the probing-promotion path are also unchanged.\n\nWhen a newly-promoted probing item is subsequently accessed in probing, ClassicLRU's `get_from_position` already moves it to `Back` (MRU), so a single re-access rescues it to full MRU protection. If it is not re-accessed, it evicts from the LRU end of probing first, which is the desired probationary behaviour.\n\n## Rationale\n\nThe cross-cycle open question \"intra-normal-tier eviction ordering: scan resistance or segmented LRU within the hot tier\" has not yet been probed. Inserting newly-promoted items at LRU of probing mimics the probationary segment of a segmented-LRU: items that were only transiently hot (promoted once but not revisited) are evicted before durable hot residents. Items that continue to be accessed earn MRU protection through ClassicLRU's move-to-back on their next probing hit. The stash→target MRU re-entry gain (from cycle/0003/v002) is fully preserved, and this change is orthogonal to both the ghost-set experiments (H01) and the demotion-path experiments (H02).\n"
}
```

### 13:22:44 chat

Three hypothesis files written to the output directory. Here's a summary:

| File | Parent | Target | Strategy |
|------|--------|--------|----------|
| **H01** | `cycle/0004/v002` | hit-rate | Change ghost re-admission insertion from `Front` → `Back` in `dash_segment.rs`. The cycle/0004/v002 insight explicitly named Front-insertion as the neutralizing factor for the ghost set's latency benefit. |
| **H02** | `cycle/0004/v002` | hit-rate | Hard-code all demotion insertions to `Front` (LRU) in `demote_from_normal_bucket`, while promotions keep `Back` (MRU). This asymmetry — promotions to MRU, demotions to LRU — is standard segmented-LRU practice and has never been tested. |
| **H03** | `cycle/0003/v002` | hit-rate | Insert newly-promoted probing entries at `Front` (LRU) instead of `Back` (MRU) in `promotion_get`. Creates a probationary zone in the probing tier: items that get one more access earn MRU protection; items that don't get filtered out. Directly addresses the "intra-normal-tier eviction ordering" open question from cross-cycle insights. |

H01 is the required incumbent citation. H02 and H03 probe two structurally independent axes — asymmetric demotion direction and probing-tier probationary insertion — neither of which has been explored in prior cycles.


---

- finished: 2026-06-05T13:22:44+00:00
- duration: 1111.28s
- status: ok
- input_tokens: 18
- output_tokens: 50605
- cache_read_tokens: 718679
- cache_write_tokens: 81874
- cost_usd: 1.43437
