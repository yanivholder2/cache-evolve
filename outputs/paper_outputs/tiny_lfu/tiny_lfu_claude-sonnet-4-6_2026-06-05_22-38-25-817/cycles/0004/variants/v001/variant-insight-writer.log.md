# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0004/variants/v001
- allow_edits: True
- started: 2026-06-06T02:03:59+00:00

## timeline

### 02:04:02 thinking

Let me read all the relevant files to understand the variant, its parent, the hypothesis, and the fitness vectors.

### 02:04:02 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0004/hypotheses/H01.md"
}
```

### 02:04:02 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0004/variants/v001/eval/fitness-vector.json"
}
```

### 02:04:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0003/variants/v003/eval/fitness-vector.json"
}
```

### 02:04:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0003/variants/v003/insight.md"
}
```

### 02:04:05 thinking

Now let me read the overlay files to see the source code changes.

### 02:04:06 chat

Now let me look at the overlay source files for both variants.

### 02:04:07 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0004/variants/v001/attempts/01/overlay"
}
```

### 02:04:07 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0003/variants/v003/attempts/01/overlay"
}
```

### 02:04:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0004/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs"
}
```

### 02:04:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0003/variants/v003/attempts/01/overlay/src/tiny_lfu/mod.rs"
}
```

### 02:04:40 thinking

Now I have all the information I need. Let me analyze what happened:

**Hypothesis**: Adding a ghost ring-buffer (capacity/4 slots) to v003 would improve hit-rate by fast-tracking re-admitted evicted keys into protected, bypassing PROMO_THRESHOLD for items the cache already knows are hot.

**What changed** (v001 vs v003):
1. Added `ghost: Vec<u64>` and `ghost_head: usize` fields to `TinyLfuCache`
2. Ghost buffer size = `(capacity / 4).max(4)` — notably larger than v002's `capacity/8`
3. Added helper functions to hash keys, insert into the ghost buffer, and check for ghost hits
4. Modified the eviction logic to record victims in the ghost buffer before removing them, and check incoming keys against the ghost to fast-track hot items directly into protected rather than probationary
5. Also checks the ghost buffer when the arena isn't full yet to fast-track new keys
6. Updated the promotion logic to bypass the threshold check if a key exists in the ghost buffer

**Fitness vectors**:
- v003 (parent): hit-rate = 0.7387
- v001 (current): hit-rate = 0.7376

So the ghost buffer approach actually decreased performance slightly.

The issue is likely that the ghost buffer is causing false positives through hash collisions with the unseeded hasher, or it's over-promoting items that were legitimately evicted because they lost frequency contests — items in the ghost are precisely those that failed to compete with incoming items, so bringing them back into protected immediately could be displacing genuinely hot entries.

Additionally, every ghost hit that fast-tracks an item into protected triggers a cascade of demotions since the protected segment is already at 90% capacity, which increases churn in the probationary segment. The ghost buffer size relative to overall capacity might also be playing a role here. is a false positive — the ghost buffer is too small or the hash function isn't selective enough to reliably distinguish between different items, so items in probationary are getting incorrectly promoted to protected based on stale ghost entries that don't actually correspond to them.

The real problem is that the ghost mechanism is promoting items that were legitimately evicted as low-frequency LRU victims back into protected when they're re-inserted. On a Zipfian trace where the hot set is stable and well-separated, this over-promotion causes hot items to get demoted out of protected to make room, increasing churn and tanking the hit rate.

### 02:04:40 chat

Now I have everything I need. Let me write the insight.

### 02:04:46 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0004/variants/v001/insight.md",
  "content": "# Variant v001 Insight\n\n## Helped\n\n## Hurt\n\n- hit-rate: The ghost ring-buffer fast-tracks re-admitted items into protected, but those items are precisely the ones that *lost* the TinyLFU admission contest (they were evicted because their frequency was lower than the incoming key's), so promoting them to protected on re-insertion displaces legitimately hot residents and increases churn in the 90% protected segment.\n\n## Takeaway\n\nAdding a capacity/4 ghost buffer on top of v003's promotion gate did not improve hit-rate on this Zipfian trace — it slightly worsened it. The flaw is that the ghost buffer records victims of the frequency-gated eviction path: these items were already identified as less-hot than their replacements, so fast-tracking them into protected upon return misidentifies a low-frequency survivor as a high-priority item, cascading unnecessary demotions out of a segment that v003's `PROMO_THRESHOLD` had already stabilized. On a trace with a clean Zipfian hot set, the `was_protected` bypass handles the legitimate \"evicted-by-overflow\" case adequately, leaving little room for a re-admission ghost mechanism to add value.\n"
}
```

### 02:04:52 chat

`insight.md` written to `/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0004/variants/v001/insight.md`.

**Summary of findings:**

- **hit-rate moved**: down (v003 parent: 0.7387 → v001: 0.7376).
- **Root cause**: The ghost buffer records victims of the TinyLFU *admission contest* — items the frequency sketch already judged as less hot than their replacements. Fast-tracking those same items into `protected` on re-insertion is counter-productive: it promotes items the cache's own admission logic had demoted, displacing legitimately hot residents and causing cascaded demotions from a protected segment that v003's `PROMO_THRESHOLD` had already stabilized.
- **Takeaway**: On this clean Zipfian workload the `was_protected` bypass already covers the real "evicted-by-overflow" case; the ghost mechanism adds no complementary signal — it just introduces misidentified promotions.


---

- finished: 2026-06-06T02:04:52+00:00
- duration: 53.16s
- status: ok
- input_tokens: 6
- output_tokens: 2760
- cache_read_tokens: 85843
- cache_write_tokens: 18456
- cost_usd: 0.137544
