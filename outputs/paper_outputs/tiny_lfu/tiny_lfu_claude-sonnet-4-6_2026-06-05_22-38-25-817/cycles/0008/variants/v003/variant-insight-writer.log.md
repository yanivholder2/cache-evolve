# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v003
- allow_edits: True
- started: 2026-06-06T02:51:55+00:00

## timeline

### 02:52:02 thinking

Let me read all the relevant files to understand the variant, its parent, and the hypothesis before writing the insight.

### 02:52:02 chat

I'll read all the relevant files in parallel to gather the information needed.

### 02:52:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/hypotheses/H03.md"
}
```

### 02:52:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v003/eval/fitness-vector.json"
}
```

### 02:52:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0007/variants/v001/eval/fitness-vector.json"
}
```

### 02:52:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0007/variants/v001/insight.md"
}
```

### 02:52:07 thinking

Now let me look at the current variant's source files to understand the actual changes made.

### 02:52:08 chat

Now let me look at the current variant's overlay source files:

### 02:52:10 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v003/attempts/01/overlay"
}
```

### 02:52:10 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v003/eval"
}
```

### 02:52:12 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v003/attempts/01/overlay/src/tiny_lfu/mod.rs"
}
```

### 02:52:13 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0007/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs"
}
```

### 02:52:38 thinking

Now I have all the information I need. Let me analyze what I've found:

**Parent (v001):**
- hit-rate summary: 0.7435
- All new items, regardless of their sketch frequency, enter probationary segment

**Current (v003):**
- hit-rate summary: 0.7424
- Change: Items whose sketch estimate at admission time is ≥ PROMO_THRESHOLD (8) bypass probationary and go directly into the protected segment — both in the fill phase and the eviction phase

**The change:**
- In the fill phase: Check `freq = self.sketch.estimate(&key)` before pushing — if `freq >= PROMO_THRESHOLD`, push to `prot_push_back` instead of `prob_push_back`
- In the eviction phase: Same logic applies — items with incoming frequency above the threshold skip probationary entirely

**Result:**
- hit-rate went DOWN from 0.7435 to 0.7424 — a slight regression

**Why did this fail?**
The hypothesis assumed many re-encountered hot keys would have sketch frequency ≥ 8 at admission time. But the optimization backfired: by filling protected slots directly, it accelerates the demotion cycle from protected back to probationary. Since protected capacity is only 90% of total (around 1843 slots at 2048 capacity), this creates a bottleneck where items get demoted faster than they should.

The core issue is that sketch estimates at admission time aren't reliable predictors of hotness — especially for cold keys or first-time encounters. When new admissions steal protected slots, they displace items that were legitimately promoted through normal `get()` operations, causing extra churn. Items demoted with `was_protected = true` can then bypass the threshold check on their next access, adding more turbulence to the cache.

I'm realizing the sketch increment happens before admission, so the frequency estimates might not be capturing the full picture of what's actually hot. The sketch collisions are likely letting false positives slip into protected, and the probationary segment actually serves as a useful filter that confirms an item's true hotness before it claims protected space. Bypassing that gate for newly-admitted items causes more churn than it saves, which explains why the hit-rate dropped — the protected segment fills with noise while legitimate residents get evicted.

### 02:52:42 chat

Now I have all the information needed. Let me write the insight.

**Summary of findings:**
- **Parent v001** hit-rate: 0.7435; all new items enter probationary unconditionally.
- **v003** hit-rate: 0.7424 — a regression. The single change: both in the fill phase and the eviction phase of `put()`, items whose sketch estimate at admission is ≥ `PROMO_THRESHOLD` (8) go directly into protected instead of probationary, followed by a `maybe_demote_from_protected()` call.
- The mechanism backfired: every direct-to-protected admission immediately triggers a demotion cascade from the protected LRU head, displacing existing protected residents earlier than the normal `get()`-driven promotion path would. The net churn in the protected segment offsets the benefit of skipping probationary for the incoming item.

### 02:52:48 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v003/insight.md",
  "content": "# Variant v003 Insight\n\n## Helped\n\n## Hurt\n\n- hit-rate: Admitting high-frequency newcomers directly into the protected segment forces an immediate `maybe_demote_from_protected()` call, which evicts the current protected LRU resident earlier than the normal `get()`-driven promotion path would — the added churn displaces items that had already earned their protected position and outruns any gain from shortcutting probationary.\n\n## Takeaway\n\nBypassing probationary at admission time for sketch-hot items proves counterproductive because the protected segment is a zero-sum space: every direct-to-protected insert immediately displaces an existing protected resident via demotion, whereas the normal flow (enter probationary, promote on first `get()`) defers that displacement until the item's hotness is reconfirmed by an actual access. The probationary gauntlet is not mere overhead for re-encountered hot items — it acts as a timing gate that prevents newly admitted items from cannibalising protected capacity before they prove continued demand on this workload.\n"
}
```

### 02:52:55 chat

`insight.md` written to the output dir. Here's what it captures:

**What happened:** v003's hit-rate fell below its parent v001's. The hypothesis predicted that directly admitting high-frequency newcomers into the protected segment would reduce churn for re-discovered hot items, but the opposite occurred.

**Why it hurt:** Every direct-to-protected admission in `put()` immediately calls `maybe_demote_from_protected()`, which evicts whichever item currently sits at the protected LRU head. The normal code path (enter probationary → promote only on a confirmed `get()`) defers that displacement until an actual access reconfirms the item's demand. By jumping straight to protected, v003 cannibalises protected capacity for items that may not receive another access, displacing residents that had already proven continued hotness via `get()`.

**Takeaway:** Probationary is not just a bureaucratic gauntlet — it is a timing filter that decouples sketch history (which reflects the past) from confirmed current demand. Removing that filter at admission time creates more protected-segment churn than it saves.


---

- finished: 2026-06-06T02:52:55+00:00
- duration: 59.53s
- status: ok
- input_tokens: 6
- output_tokens: 2724
- cache_read_tokens: 88014
- cache_write_tokens: 20546
- cost_usd: 0.145493
