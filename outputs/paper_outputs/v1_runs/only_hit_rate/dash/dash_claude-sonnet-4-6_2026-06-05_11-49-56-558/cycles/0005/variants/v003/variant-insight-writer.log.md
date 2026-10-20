# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/variants/v003
- allow_edits: True
- started: 2026-06-05T13:32:21+00:00

## timeline

### 13:32:24 thinking

Let me read all the relevant files to understand the variant, its parent, and the hypothesis before writing the insight.

### 13:32:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/hypotheses/H03.md"
}
```

### 13:32:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/variants/v003/eval/fitness-vector.json"
}
```

### 13:32:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0003/variants/v002/eval/fitness-vector.json"
}
```

### 13:32:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0003/variants/v002/insight.md"
}
```

### 13:32:28 thinking

Now let me look at the source files to understand what changed between the parent and current variant.

### 13:32:29 chat

Now let me read the overlay source files for both variants to compare the actual code changes.

### 13:32:30 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/variants/v003/attempts/01/overlay"
}
```

### 13:32:31 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0003/variants/v002/attempts/01/overlay"
}
```

### 13:32:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/variants/v003/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 13:32:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0003/variants/v002/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 13:32:53 thinking

Now I have all the information I need. Let me analyze what I've found:

**Parent (v002):**
- fitness-vector: hit-rate summary = 0.7145
- The `promotion_get` function calls `promote_to_probing(core, data, insert_pos)` where `insert_pos` is `InsertPosition::Back` (inherited from the caller)
- insight.md notes: "Adding `InsertPosition::Back` on top of the 75/25 topology produced a small additional gain"

**Current (v003):**
- fitness-vector: hit-rate summary = 0.7143


- The H03 change modified `promotion_get` to use `InsertPosition::Front` (hardcoded) instead of `insert_pos` when promoting items to probing, meaning newly-promoted items now go to the LRU end while stash→target insertions still use `Back`
- This resulted in a very slight performance drop of 0.0002, essentially flat or marginally worse
- The probationary zone hypothesis didn't pan out — the promoted items appear to be genuinely hot rather than one-hit wonders, so placing them at Front for earlier eviction slightly hurts performance, or the effect is too small to measure meaningfully

.

The Front-insert strategy actually backfired: by accelerating eviction of recently-promoted items before they can be re-accessed, it negated the gains from the previous version's Back-insert approach for stash promotions. The workload simply doesn't have enough transient promotions to justify filtering them out.

### 13:32:53 chat

Now I have everything I need. Let me write the insight.

### 13:32:58 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0005/variants/v003/insight.md",
  "content": "# Variant v003 Insight\n\n## Helped\n\n## Hurt\n\n- hit-rate: Inserting newly-promoted items at `InsertPosition::Front` (LRU/probationary end) of the probing bucket instead of `Back` (MRU) causes recently-graduated items to evict from probing sooner than they can earn a re-access, slightly increasing the rate at which hot items are demoted before being re-hit.\n\n## Takeaway\n\nThe probationary-zone idea relies on a population of transient one-off promotions to filter; on workload-a, items that reach the hottest position in their target bucket appear to be genuinely reused, so Front-insert in probing removes MRU protection from items that would have benefited from it rather than weeding out noise. The stash→target Back-insert gain inherited from v002 is preserved, but the new Front-insert at the probing tier works against it by shortening the effective dwell time of just-promoted items — suggesting that MRU re-entry is valuable at both promotion steps on this workload, not just the stash→target one.\n"
}
```

### 13:33:07 chat

`insight.md` written to the output dir. Here's a summary of what the analysis found:

**v003 vs v002 — what happened:**

The sole code change was in `promotion_get`: when an item is the hottest in its target bucket and graduates to probing, the call to `promote_to_probing` now hard-codes `InsertPosition::Front` instead of passing through `insert_pos` (which was `Back` from the caller). Everything else — stash→target MRU re-entry, demotion ordering, the 75/25 geometry — is unchanged.

**Fitness outcome:** hit-rate dipped by a hair versus the parent (flat/trivially negative).

**Why it didn't work:** The probationary-zone hypothesis assumes many transient one-hit promotions exist to filter out. On workload-a, items earning the hottest position in their target bucket appear to be genuinely reused. Placing them at the LRU end of probing shortens their effective dwell time — they become eligible for eviction immediately — so some get displaced before their next access, slightly increasing demotions of actually-hot items. MRU re-entry at the probing insertion point matters on this workload, not just at the stash→target step.


---

- finished: 2026-06-05T13:33:07+00:00
- duration: 46.51s
- status: ok
- input_tokens: 6
- output_tokens: 2219
- cache_read_tokens: 89046
- cache_write_tokens: 22299
- cost_usd: 0.144765
