# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/variants/v002
- allow_edits: True
- started: 2026-06-05T14:48:54+00:00

## timeline

### 14:48:57 thinking

Let me read all the relevant files to understand the variant, its parent, and the hypothesis before writing the insight.

### 14:48:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/hypotheses/H02.md"
}
```

### 14:48:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/variants/v002/eval/fitness-vector.json"
}
```

### 14:48:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/eval/fitness-vector.json"
}
```

### 14:48:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/insight.md"
}
```

### 14:49:02 thinking

Now let me look at the current variant's overlay and the parent's overlay to see the actual code changes.

### 14:49:02 chat

Now let me look at the code changes in the overlays:

### 14:49:04 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/variants/v002/attempts/01/overlay"
}
```

### 14:49:04 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay"
}
```

### 14:49:08 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/variants/v002/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 14:49:08 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 14:49:25 thinking

Now I have a clear picture of what changed. Let me analyze:

**The change**: In `put()`, when stash overflows and a resident item is evicted, the second-chance promotion changed from:
- v001 (parent): `promote_to_target(&mut self.core, ev, InsertPosition::Front)` — LRU-end insertion
- v002 (current): `promote_to_target(&mut self.core, ev, InsertPosition::Back)` — MRU-end insertion

**Fitness results**:
- v001 hit-rate: 0.7222


- v002 hit-rate: 0.7149

So hit-rate dropped with MRU-end insertion. My hypothesis that placing second-chance items at the hot end would improve performance was wrong. It seems LRU insertion works better because second-chance items haven't proven themselves hot yet—they were displaced from the cold stash tier—so inserting them at the front preserves the existing hot items from being pushed toward eviction.

The real problem with MRU insertion is that it treats "still in cache" as equivalent to "actually hot." Second-chance items immediately become eligible for promotion on their next access, which floods the upper tier with unproven candidates and crowds out genuinely hot items that earned their position through repeated accesses. Front insertion acts as a natural filter by requiring two target accesses before the item becomes a promotion candidate, preventing these stash refugees from consuming probing slots too aggressively.

### 14:49:25 chat

Now I have everything I need. Let me write the insight:

### 14:49:31 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/variants/v002/insight.md",
  "content": "# Variant v002 Insight\n\n## Helped\n\n## Hurt\n\n- hit-rate: Changing the second-chance promotion from `InsertPosition::Front` to `InsertPosition::Back` placed stash-evicted items immediately at the MRU end of their target bucket, making them instant candidates for probing promotion and displacing genuinely hot items that had earned their target position through repeated access.\n\n## Takeaway\n\nThe MRU-wins principle does not extend to second-chance items: they are stash residents displaced by hash collision, not by proven coldness, but they have also not demonstrated hotness. Inserting them at Back gives them immediate probing candidacy on their very next access, which floods the probing tier with unproven items and evicts residents that earned their position through actual repeated use. Front insertion, as used in v001, correctly treats second-chance items as cold probationers that must survive target competition before advancing — the asymmetry between v001 and v002 confirms that entry position at the second-chance recovery site acts as a quality gate, not just a placement detail.\n"
}
```

### 14:49:39 chat

The `insight.md` has been written to the output directory. Here's a summary of what it captures:

**What changed**: v002 switched the second-chance stash-overflow promotion from `InsertPosition::Front` (LRU end) to `InsertPosition::Back` (MRU end) — a single-line diff in `put()`.

**What happened**: Hit-rate fell compared to v001. The hypothesis predicted MRU insertion would help because other promotion paths use Back insertion, but the opposite occurred.

**Why**: Second-chance items are stash residents evicted by hash collision — they're cold by circumstance, not hot by access frequency. Landing them at Back makes them immediately eligible for probing promotion on the very next hit, which floods the probing tier with unproven items and pushes out residents that genuinely earned their positions. Front insertion (v001) acts as a quality gate: second-chance items must survive target competition across two accesses before advancing, preserving probing slots for proven-hot items. The MRU-wins principle applies to demonstrated hotness, not to survivor-by-luck items.


---

- finished: 2026-06-05T14:49:39+00:00
- duration: 44.82s
- status: ok
- input_tokens: 6
- output_tokens: 2111
- cache_read_tokens: 90828
- cache_write_tokens: 24274
- cost_usd: 0.151076
