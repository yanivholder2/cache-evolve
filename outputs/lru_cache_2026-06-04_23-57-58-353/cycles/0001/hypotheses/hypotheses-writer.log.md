# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/hypotheses
- allow_edits: True
- started: 2026-06-04T20:57:58+00:00

## timeline

### 20:58:02 thinking

Let me read the context files to understand the codebase and then generate 3 distinct hypotheses.

### 20:58:02 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/manifest.toml"
}
```

### 20:58:03 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/eval/fitness-vector.json"
}
```

### 20:58:03 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/insight.md"
}
```

### 20:58:06 tool: Glob

```json
{
  "pattern": "**/*.rs",
  "path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src"
}
```

### 20:58:08 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/lru_cache/mod.rs"
}
```

### 20:58:08 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/test_utils/invariants.rs"
}
```

### 20:59:30 thinking

Now I have a good understanding of the codebase. Let me analyze what I've learned:

1. **Baseline**: O(1) LRU cache backed by a `HashMap` + arena-based doubly-linked list. Single fitness dimension: **hit-rate = 0.6652** on workload-a.trace.gz at size 2048.

2. **Structure**: 
   - `LruCache<K, V>` with a `map: HashMap<K, usize>` and `nodes: Vec<Node<K, V>>`
   - `head` = LRU (front), `tail` = MRU (back)
   - Arena slots get reused in-place when the cache reaches capacity, with the LRU slot being recycled for new entries

3. **Key operations**:
   - `get`: look up arena index, unlink from current position, push_back to MRU tail
   - `put`: if exists, update value and promote to MRU; if new, either allocate a slot or evict LRU
   - `unlink`: detach a node from the doubly-linked list
   - `push_back`: append to tail (MRU)

4. **Invariants to preserve**:
   - P1: capacity_bound — never more than `capacity` keys
   - P2: value_fidelity — resident values stay consistent
   - P3: no_phantom_items — keys never inserted always return None

5. **Fitness**: hit-rate = 0.6652 (higher is better), only one dimension

Now I'm exploring three distinct strategies to boost hit-rate on the workload-a trace. The core challenge is finding eviction policies or structural changes that outperform basic LRU. I'm considering SLRU as a first approach—splitting the cache into probationary and protected segments so items seen multiple times get preferential treatment, which typically handles scan-heavy workloads better than pure LRU. Items that get accessed again graduate from probationary to protected, reducing eviction of frequently-used entries.

For a second strategy, I'm thinking about LRU-K, which tracks multiple access timestamps per item and evicts based on the older historical accesses rather than just the most recent one. This dampens the effect of one-time scans that would otherwise pollute the cache. Then there's CLOCK with a second-chance mechanism—adding a referenced bit so items get a reprieve on eviction if they've been touched recently, which works well for cyclic patterns. I'm also considering a frequency-based approach as a fourth option. sted LRU (tiny LFU approximation)**
Add a small frequency counter per item. On eviction among candidates, prefer to evict the less-frequently-accessed item. This mixes LRU recency with LFU frequency.

**Approach 5: ARC (Adaptive Replacement Cache)**
A more complex but well-known improvement. Maintain two LRU lists (T1 for items seen once, T2 for items seen multiple times) and two ghost lists (B1, B2). Adaptively tune the split between T1 and T2 based on hit patterns. This is complex but well-validated.

**Approach 6: LIRS-like "hot/cold" classification**
Since the trace is in LIRS format, it likely benefits from LIRS-style policies that classify items as "hot" or "cold" based on inter-reference recency, protecting a small resident hot set while allowing cold items to be evicted.

Given the LIRS-formatted trace and the baseline LRU's 66.52% hit rate, I'm narrowing down to three distinct strategies. SLRU or 2Q-style segmentation divides capacity into protected and probationary regions, promoting items that hit twice and avoiding pollution from one-time accesses. CLOCK second-chance approximates LRU more efficiently by giving referenced items a second chance before eviction rather than immediately removing them.

For the third, I could go with frequency-aware eviction—keeping the LRU structure but adding a small counter per node so that when evicting, I scan a small window from the LRU end and remove the least-frequently-accessed item, protecting hot items that happen to land at the tail.

Actually, since I have full autonomy over the implementation as long as the JNI ABI stays intact, I can make more substantial changes. Let me think through some concrete options.

The SLRU approach splits the cache into two segments: a protected region for items accessed at least twice (taking up about 80% of capacity) and a probationary region for new items (20%). On a get hit in probationary, I promote it to protected and evict the LRU item from protected back to probationary if needed. On a put miss, I insert into probationary as most-recent, and if that fills up, I evict the least-recent item entirely. This is well-proven to beat pure LRU, especially against scan patterns.

For CLOCK, I'd replace the linked list with a circular arena where each slot has a referenced bit. On access, I set that bit, and during eviction I sweep the clock hand forward—if the bit is set, I clear it and move on; if it's clear, I evict that slot.

LRU-K tracks multiple access timestamps per item and evicts based on the second-most-recent access time instead of the most recent. Items seen only once get a -∞ timestamp so they're always evicted first. I could implement this coarsely with just a frequency bit: items accessed once are candidates first, then among those I evict by LRU order, and among multiply-accessed items I do the same.

Actually, that's just SLRU in disguise. Let me think about truly different strategies.

ARC maintains four lists—T1 for single accesses, T2 for frequent accesses, plus ghost lists B1 and B2 tracking recently evicted items—and adaptively rebalances capacity between them based on ghost hits. It's self-tuning but complex.

Or I could try something LIRS-inspired: keep a small ghost list of recently evicted keys capped at half capacity, and on a miss check if the key's in there. If it is, that's a "warm" miss and I insert it partway through the cache rather than at the front.

Let me step back and think about what actually matters here. The core strategies worth comparing are SLRU with its protected and probationary segments, CLOCK as a simpler LRU approximation, and LRU with an admission filter using a ghost buffer to prevent scan pollution. I need to clarify what "full autonomy" really means in this context.

For the implementation details: SLRU needs separate size parameters and two linked list segments within the same arena. CLOCK replaces the doubly-linked list with a clock-hand pointer and adds a referenced bit to each node, where gets just set the bit without moving anything, and eviction sweeps forward clearing bits. The ghost buffer approach tracks recently-evicted keys in a small ring-buffer to distinguish first-time insertions from re-insertions, though I'm second-guessing whether inserting new keys at the head actually helps or hurts performance.

I'm reconsidering the ghost buffer strategy—maybe instead of changing insertion position, I should track keys that were evicted once and promote them back to a protected segment on re-insertion. Or I could try a lazy promotion approach where gets only move items to MRU if they're not already in the hot zone, reducing list churn while naturally extending the lifespan of cold-zone items.

The three strongest approaches seem to be SLRU with protected and probationary zones for scan resistance, CLOCK with a referenced bit and clock-hand eviction for better cache locality, or a frequency-aware hybrid that adds a 2-bit counter per node and evicts the lowest-frequency candidate from the LRU tail. For the frequency approach, I need to think through the implementation details.

Each node would track a saturating frequency counter that increments on access and resets to zero on insertion. During eviction, instead of removing the head immediately, I'd scan a small window of k nodes from the LRU end and evict whichever has the lowest frequency, breaking ties by recency. This gives me a lightweight approximation of TinyLFU without the overhead of a full frequency table.

Now I'm ready to formalize these three hypotheses with enough specificity that someone could implement them in Rust.

For the first hypothesis, I'm proposing a segmented LRU design that splits the cache into a protected region (about 4/5 of capacity) and a probationary region. New entries land in probationary and get promoted to protected on a second access, which should help filter out one-time scans. The implementation would use separate head/tail pointers for each segment and track which segment each node belongs to, with promotion logic on cache hits and independent eviction policies for each region.

For the second hypothesis, I'm exploring a CLOCK algorithm that replaces the doubly-linked list with a circular sweep using a clock hand pointer and a referenced flag on each node, giving entries a second chance to be accessed before eviction. - Remove `prev`/`next` from `Node`; instead the clock sweeps through the arena sequentially (slot indices 0..capacity wrap around)
- `get`: look up arena index, set `nodes[idx].referenced = true`; no structural movement
- `put` (new key): if arena is not full, allocate the next slot and advance clock_hand. If full, sweep from `clock_hand`: if `nodes[clock_hand].referenced` is true, clear it and advance; otherwise evict `clock_hand` slot (remove from map, overwrite with new entry, advance hand)
- `put` (existing key): update value in place, set `referenced = true`
- Invariants: capacity_bound still enforced (at most one new entry per put); value_fidelity and no_phantom_items preserved.

Now for the frequency-tiebreak approach: I'm adding a frequency counter to each node that saturates at 3, incremented on every access. When eviction is needed, instead of always removing the oldest entry, I'll scan a small window of 8 candidates from the LRU end and evict whichever has the lowest frequency, protecting hot items from being displaced during cold-start phases.

Writing the H01.md, H02.md, H03.md files...

### 20:59:30 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/src/lib.rs"
}
```

### 20:59:39 thinking

Now I have all the context I need. Let me write the 3 hypothesis files.

Key facts:
- Baseline hit-rate: **0.6652** at capacity 2048
- The trace is a LIRS workload (workload-a.trace.gz)
- The implementation is a pure LRU using HashMap + doubly-linked list arena
- Only `src/lru_cache/lru_cache_jni.rs` is protected; I can modify `mod.rs` (the main impl)
- Parent for all 3 must be `baseline`


- I'm targeting three distinct strategies: SLRU with probationary/protected segmentation for scan resistance, CLOCK second-chance using a clock-hand and referenced bits instead of linked list operations, and a window frequency tiebreak approach that scans the LRU-end window to evict the least-frequently-accessed item.

### 20:59:39 chat

Now I have all the context needed. Let me write the three hypothesis files.

### 20:59:40 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/hypotheses 2>/dev/null || echo \"directory does not exist yet\"",
  "description": "Check whether the output directory exists"
}
```

### 20:59:57 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/hypotheses/H01.md",
  "content": "# H01\n\nReplacing the single LRU list with a two-segment SLRU layout (probationary + protected) will raise hit rate by shielding frequently-reused entries from one-hit scan pollution.\n\n## Parent\n\nbaseline\n\n## Changes\n\nRestructure `LruCache` in `src/lru_cache/mod.rs` to maintain two ordered sub-lists within the same arena and `HashMap`, with separate head/tail pointers for each segment:\n\n**New fields on `LruCache`:**\n- `prot_head: Option<usize>`, `prot_tail: Option<usize>` — protected segment (LRU end / MRU end)\n- `prob_head: Option<usize>`, `prob_tail: Option<usize>` — probationary segment\n- `prot_count: usize`, `prob_count: usize` — current occupancy of each segment\n- `prot_cap: usize` — set to `(capacity * 4).div_ceil(5)` in `try_new` (80 % of total)\n\nRemove the current unified `head`/`tail` fields.\n\n**New field on `Node`:**\n- `in_protected: bool` — true when the node lives in the protected segment\n\n**`put` (new key):**\nInsert the new node at the probationary MRU end (`prob_tail`). Set `in_protected = false`. Increment `prob_count`. If `prob_count > capacity - prot_cap`, evict `prob_head` (remove its key from the map, mark slot free with a sentinel or reuse it for the incoming entry as the baseline does).\n\n**`get` (probationary hit):**\nUnlink the node from the probationary list, decrement `prob_count`. Link it at the protected MRU end (`prot_tail`), set `in_protected = true`, increment `prot_count`. If `prot_count > prot_cap`, demote `prot_head`: unlink it from protected (decrement `prot_count`), link it at the probationary MRU end (increment `prob_count`), set `in_protected = false`. If `prob_count` now exceeds its cap, evict `prob_head` as above.\n\n**`get` (protected hit):**\nUnlink the node from its current position in the protected list and relink at `prot_tail` (standard MRU promotion within the segment). Counts do not change.\n\n**`put` (existing key, either segment):**\nUpdate value in place, then apply the same promotion logic as the corresponding `get` case above (probationary hit promotes; protected hit moves to protected MRU).\n\n**`unlink` / `push_back`:**\nKeep the existing `unlink` and `push_back` helpers but add `push_back_prob` and `push_back_prot` variants that update the correct head/tail pair. `unlink` remains segment-agnostic (it reads `prev`/`next` from the node, which are always correct).\n\n**Invariants:**\nTotal resident keys = `prot_count + prob_count <= capacity` at all times, satisfying P1. Value writes happen before any structural move, preserving P2 and P3.\n"
}
```

### 21:00:10 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/hypotheses/H02.md",
  "content": "# H02\n\nReplacing doubly-linked-list eviction with a CLOCK second-chance sweep will raise hit rate by granting one access-reprieve to entries that were touched after entering the LRU end of the cache.\n\n## Parent\n\nbaseline\n\n## Changes\n\nRewrite the eviction and promotion machinery in `src/lru_cache/mod.rs` to use a circular clock instead of a moving-pointer doubly-linked list.\n\n**Remove from `LruCache`:**\n- `head`, `tail` fields\n- `unlink`, `push_back` methods\n\n**Add to `LruCache`:**\n- `clock_hand: usize` — index of the next candidate slot to inspect during eviction; starts at 0; wraps modulo `capacity`\n- `len: usize` — current number of resident entries (replaces the implicit `nodes.len() < capacity` check)\n\n**Change on `Node`:**\n- Remove `prev: Option<usize>` and `next: Option<usize>`\n- Add `referenced: bool` — cleared to `false` on insertion, set to `true` on any subsequent `get` or `put`-update\n\n**`get`:**\nLook up `idx` in `map`. If present, set `nodes[idx].referenced = true`; return `Some(&nodes[idx].value)`. No structural movement.\n\n**`put` (existing key):**\nUpdate `nodes[idx].value`. Set `nodes[idx].referenced = true`. Return.\n\n**`put` (new key, arena not full):**\nPush a new `Node` with `referenced = false` onto `nodes`. Increment `len`. Insert into `map`.\n\n**`put` (new key, arena full — eviction path):**\nSweep forward from `clock_hand`:\n```\nloop {\n    if nodes[clock_hand].referenced {\n        nodes[clock_hand].referenced = false;  // give second chance\n        clock_hand = (clock_hand + 1) % capacity;\n    } else {\n        // evict this slot\n        map.remove(&nodes[clock_hand].key);\n        nodes[clock_hand] = Node { key: key.clone(), value, referenced: false };\n        map.insert(key, clock_hand);\n        clock_hand = (clock_hand + 1) % capacity;\n        break;\n    }\n}\n```\nThe sweep always terminates because at most one full revolution clears all reference bits, guaranteeing an unreferenced victim exists.\n\n**Invariants:**\n`len` never exceeds `capacity` (exactly one slot is reused per eviction), so P1 holds. Values are written atomically before the map is updated, preserving P2 and P3.\n"
}
```

### 21:00:26 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/hypotheses/H03.md",
  "content": "# H03\n\nScanning a window of 8 LRU-end candidates and evicting the one with the lowest saturating access-frequency counter will raise hit rate by preventing hot entries from being displaced during cold-start bursts.\n\n## Parent\n\nbaseline\n\n## Changes\n\nExtend the existing LRU implementation in `src/lru_cache/mod.rs` with a per-node frequency counter and a small-window victim-selection pass at eviction time. All other logic (arena reuse, `unlink`, `push_back`, map management) is unchanged.\n\n**Add to `Node<K, V>`:**\n```rust\nfreq: u8,  // saturating counter, max value 3; 0 on construction\n```\n\n**`get` hit:**\nAfter the existing `self.unlink(idx); self.push_back(idx);` calls, add:\n```rust\nself.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1).min(3);\n```\n\n**`put` (existing key):**\nAfter updating the value and promoting the node, apply the same saturating increment to `self.nodes[idx].freq`.\n\n**`put` (new key):**\nWhen constructing the new `Node`, set `freq: 0`.\n\n**Eviction path in `put` (arena full):**\nReplace the direct `let head_idx = self.head.expect(...)` victim selection with a window scan:\n\n```rust\nconst WINDOW: usize = 8;\nlet mut victim = self.head.expect(\"non-empty list has a head\");\nlet mut cursor = victim;\nfor _ in 1..WINDOW {\n    match self.nodes[cursor].next {\n        Some(next_idx) => {\n            if self.nodes[next_idx].freq < self.nodes[victim].freq {\n                victim = next_idx;\n            }\n            cursor = next_idx;\n        }\n        None => break,\n    }\n}\n// victim is now the lowest-frequency node among the first WINDOW LRU-end entries\nlet evicted_key = self.nodes[victim].key.clone();\nself.map.remove(&evicted_key);\nself.unlink(victim);\nself.nodes[victim].key = key.clone();\nself.nodes[victim].value = value;\nself.nodes[victim].freq = 0;\n// then push_back(victim) and map.insert(key, victim) as before\n```\n\nWhen `capacity < WINDOW` the loop simply exhausts the list and picks the best available victim, so the logic is correct for any capacity including 1.\n\n**Invariants:**\nExactly one slot is evicted per new-key insertion, so `map.len() <= capacity` (P1) is maintained. Values are written to the victim slot before it is re-linked and re-inserted into the map, so P2 and P3 are unaffected.\n"
}
```

### 21:00:39 chat

Three hypotheses written to the output directory. Here is a summary of what each proposes:

---

**H01 — Segmented LRU (SLRU / 2Q)**
Splits capacity into a *protected* zone (80 %) and a *probationary* zone (20 %) within the same arena. New entries land in probationary; a `get` hit on a probationary item promotes it to the protected MRU end. Overflow from protected demotes the protected LRU entry back to probationary before evicting the probationary LRU. This is the canonical scan-resistance fix for workloads where one-time accesses dilute the resident hot set.

**H02 — CLOCK second-chance**
Drops the doubly-linked list entirely. Each arena slot carries a `referenced` bit instead of `prev`/`next` pointers. `get` and `put`-updates set the bit; structural movement stops. Eviction sweeps a circular clock hand: referenced slots get their bit cleared and are skipped; the first unreferenced slot is evicted. Gives every accessed entry one extra sweep of protection with zero list-pointer writes.

**H03 — LRU-end window frequency tiebreak (QLRU-W)**
Keeps the existing LRU structure intact but adds a 2-bit saturating frequency counter per node. On eviction, instead of blindly evicting the head, it scans the 8 oldest entries and evicts whichever has the lowest frequency (LRU order breaks ties). This mixes recency with a lightweight frequency signal, protecting items that were recently accessed multiple times even if they drifted toward the LRU end.


---

- finished: 2026-06-04T21:00:39+00:00
- duration: 161.45s
- status: ok
- input_tokens: 10
- output_tokens: 7794
- cache_read_tokens: 186172
- cache_write_tokens: 16888
- cost_usd: 0.237022
