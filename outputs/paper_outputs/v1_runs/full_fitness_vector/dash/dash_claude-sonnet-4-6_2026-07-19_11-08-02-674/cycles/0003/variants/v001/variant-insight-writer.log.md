# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\variants\v001
- allow_edits: True
- started: 2026-07-19T09:17:25+00:00

## timeline

### 09:17:33 thinking

Let me read all the relevant files to understand the variant, its parent, and the hypothesis before writing the insight.

### 09:17:34 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\hypotheses\\H01.md"
}
```

### 09:17:35 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v001\\eval\\fitness-vector.json"
}
```

### 09:17:36 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\eval\\fitness-vector.json"
}
```

### 09:17:36 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\insight.md"
}
```

### 09:17:39 thinking

Now let me look at the current variant's overlay (changed source files) and the parent's overlay to understand the diff:

### 09:17:39 chat

Now let me examine the current and parent overlays to understand what code changes were actually made:

### 09:17:41 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v001\\attempts\\01\\overlay"
}
```

### 09:17:42 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\attempts\\01\\overlay"
}
```

### 09:17:45 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v001\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 09:17:46 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0002\\variants\\v001\\attempts\\01\\overlay\\src\\dash\\segments\\dash_segment.rs"
}
```

### 09:18:08 thinking

Now I have all the information I need. Let me analyze the changes and the fitness results:

**Parent (cycle/0002/v001):**
- Had MRU-end insertion (`InsertPosition::Back`) + compact 24-byte Item (timestamp removed)
- hit-rate: 0.6787
- metadata-size: 24 bytes
- per-op-time p50: 100ns, p99: 200ns

**Current (cycle/0003/v001):**
- Added saturation-aware stash routing on top of the parent
- hit-rate: 0.6783
- metadata-size: 24 bytes
- per-op-time p50: 100ns, p99: 300ns

The hypothesis is that routing stash hits intelligently based on target saturation should boost hit rate by preventing cascade evictions when the target bucket is full. The key change in the `promotion_get` function is that instead of always promoting stash hits directly to the target, it now checks if the target bucket is saturated first — if it is, the hit gets routed to probing instead.

New tests were added to verify this behavior, and the results show the change actually hurt performance: hit rate dropped slightly from 0.6787 to 0.6783, p99 latency increased from 200ns to 300ns, while metadata size stayed the same. The issue is that bypassing the target when it's full causes stash hits to land in the shared probing bucket, which then evicts other hot items through cascade eviction — defeating the original goal. uration causes cascade eviction was already true without the bypass. Bypassing to probing just shifts the cascade to a hotter tier, potentially evicting more valuable items. The benefit from the LRU-end base doesn't transfer here because with Back insertion, items arrive at MRU and promote more easily, making target saturation more common and causing the bypass to displace hot probing residents more often. Now I'm writing up the insight.

### 09:18:08 chat

Now I have all the information needed. Let me write the insight:

### 09:18:14 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0003\\variants\\v001\\insight.md",
  "content": "# Variant v001 Insight\n\n## Helped\n\n*(no dimensions moved positively)*\n\n## Hurt\n\n- **hit-rate**: Bypassing a saturated target to place stash-promoted items directly into probing displaces hot probing residents via cascade eviction, producing a net loss on this workload — the cascade from a forced probing insert is costlier than the cascade that would have occurred on a full-target insert.\n- **per-op-time**: The bypass path runs `promote_to_probing` instead of `promote_to_target`, which triggers a longer demotion chain (probing evictee → one-more-chance retry into true target → potential second eviction into stash), inflating p99 latency even though p50 held steady.\n\n## Takeaway\n\nThe saturation-aware bypass does not compose additively with MRU-end insertion: because `InsertPosition::Back` causes stash-promoted items to land immediately at the MRU end, target saturation is more common, so the bypass fires frequently and repeatedly pushes stash items into the already-contested probing tier. The benefit seen in the LRU-end baseline (cycle/0002/v003) likely came from the bypass being rare; on the MRU-end base it fires often enough to make probing a spillover sink for stash traffic, which hurts both hit rate and tail latency.\n"
}
```

### 09:18:23 chat

`insight.md` written to the output dir. Here's a summary of what the variant teaches:

**What changed:** `promotion_get` in `dash_segment.rs` gained a `is_full()` check on the target bucket at stash-hit time. When saturated, it routes the item directly to probing (`promote_to_probing`) instead of forcing it into a full target (`promote_to_target`).

**What happened:**
- **hit-rate** nudged slightly downward — the bypass fires frequently with MRU-end insertion (because Back placement means target fills faster), and each bypass pushes a stash item into the contested probing tier, which cascades out an existing hot resident. The cure is worse than the disease on this workload.
- **per-op-time** p99 grew (200 → 300 ns) — the probing path runs a longer demotion chain (probing evictee → one-more-chance retry into its true target → potential second eviction to stash), raising worst-case latency even though median was unchanged.
- **metadata-size** was unaffected — the change was pure control flow.

**Takeaway:** The bypass benefit observed on the LRU-end baseline (cycle/0002/v003) doesn't transfer to the MRU-end base. With Back insertion, target saturation is common enough that the bypass fires constantly, turning probing into a stash-traffic spillover and hurting both hit rate and tail latency.


---

- finished: 2026-07-19T09:18:23+00:00
- duration: 58.15s
- status: ok
- input_tokens: 6
- output_tokens: 2506
- cache_read_tokens: 101784
- cache_write_tokens: 23939
- cost_usd: 0.159084
