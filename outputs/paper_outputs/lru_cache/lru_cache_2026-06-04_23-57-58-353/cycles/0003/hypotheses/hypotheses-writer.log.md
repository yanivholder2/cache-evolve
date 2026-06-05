# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/hypotheses
- allow_edits: True
- started: 2026-06-04T21:41:42+00:00

## timeline

### 21:41:44 thinking

Let me read all the relevant context files to understand the current state of the research and what hypotheses have been explored so far.

### 21:41:45 chat

I'll start by reading all the relevant context files in parallel.

### 21:41:46 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/variants/*/"
}
```

### 21:41:46 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/cycle-winners.json"
}
```

### 21:41:47 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/cycle-insight.md"
}
```

### 21:41:47 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/leaderboard.md"
}
```

### 21:41:48 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles-ledger.jsonl"
}
```

### 21:41:50 thinking

Now let me read the variant overlays and insights from cycle 0002. First, let me check what's in the variants directory.

### 21:41:51 chat

Now let me read the variant details from cycle 0002 and the cross-cycle insights:

### 21:41:52 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/variants/**"
}
```

### 21:41:52 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cross-cycle-insights.md"
}
```

### 21:41:55 thinking

Now let me read the actual overlay files for the cycle 0002 variants, especially v003 (the winner) and also the insights for each variant:

### 21:41:56 chat

Now let me read the variant overlays and insights in parallel:

### 21:41:57 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/variants/v003/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 21:41:58 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/variants/v003/insight.md"
}
```

### 21:41:59 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/variants/v001/insight.md"
}
```

### 21:41:59 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/variants/v002/insight.md"
}
```

### 21:42:02 thinking

Now let me also read the baseline to understand what the original code looks like, and the cycle 0001 variants to understand the complete history:

### 21:42:02 chat

Now let me read the baseline and cycle 0001 winner for full context:

### 21:42:03 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/inputs/subjects/lru_cache/baseline/**"
}
```

### 21:42:05 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0001/variants/v001/attempts/01/overlay/src/lru_cache/mod.rs"
}
```

### 21:43:38 thinking

Now I have a comprehensive understanding of the evolution so far. Let me synthesize what I know to craft 3 distinct hypotheses for cycle 0003:

**What we know so far:**

1. **Baseline (plain LRU):** hit-rate = 0.665
2. **Cycle 0001 v001 (SLRU 80/20):** hit-rate = 0.717 — the two-segment structure is a big win
3. **Cycle 0001 v002 (CLOCK):** hit-rate = 0.675


4. **Cycle 0001 v003 (Frequency window scan):** hit-rate = 0.667
5. **Cycle 0002 v001 (70/30 SLRU ratio):** hit-rate = 0.713 — adjusting the split ratio actually decreased performance
6. **Cycle 0002 v002 (freq-aware demotion, 4-node scan, reset-on-demotion):** hit-rate = 0.717 — frequency tracking didn't help
7. **Cycle 0002 v003 (SLRU + ghost admission, from baseline):** hit-rate = 0.721 — currently the best approach

The ghost admission layer is clearly the key differentiator, pushing us past the 0.717 ceiling that pure SLRU hits. Now I need to figure out what combination of techniques can push even further.

I'm planning three focused experiments. First, I want to isolate whether ghost admission alone accounts for v003's gains by applying it to the cycle-1 SLRU baseline without any other modifications. Second, I should revisit frequency-aware demotion with a corrected design that uses persistent counters and a larger scan window. Third, I need to test different ghost capacity sizes to see if half the probationary capacity performs better than matching it fully.

At least one of these experiments has to build directly on v003 as the parent to maintain continuity with what's working. (cycle/0001/v001) will improve hit-rate by capturing cross-residency warmth independently of the baseline SLRU implementation—this isolates the ghost set's marginal contribution since cycle/0001/v001 already has the SLRU structure.

**H02: Frequency-aware demotion with persistent counters and wider scan (from cycle/0002/v003)**
- Parent: `cycle/0002/v003` (current winner)
- Change: Add persistent frequency counters to protected nodes (do not reset on demotion); replace fixed 4-node scan with a full-segment scan on overflow; evict the node with the lowest access count rather than the strict LRU head
- Claim: Persistent frequency counters and a wider scan window inside the protected segment will improve hit-rate by enabling durable discrimination between high- and low-reuse entries, addressing the two specific failure modes identified in cycle 2

**H03: Smaller ghost window (from cycle/0002/v003)**
- Parent: `cycle/0002/v003` (current winner)
- Change: Reduce ghost_cap from prob_cap to prob_cap/2 (half the probationary capacity)
- Claim: Halving the ghost window to prob_cap/2 preserves most of v003's cross-residency benefit while reducing ghost set bookkeeping overhead, and may improve effective filtering by keeping only the most recently evicted keys

I'm reconsidering whether H03 is strong enough—it's a minor variation. Looking back at the cross-cycle insights, I notice the CLOCK throughput advantage comes from eliminating linked-list pointer overhead, which suggests a different direction worth exploring.

Since we're measuring hit-rate specifically, I should consider strategies that adapt the protected/probationary split dynamically based on segment performance, track both probationary and protected evictions separately to distinguish warm from cold keys, or refine how frequency information guides demotion decisions within the protected segment.

For testing, I'll focus on three distinct approaches: first, adding ghost admission to the baseline SLRU to isolate its contribution; second, enhancing the frequency demotion strategy with persistent counters that survive across demotions and scanning a larger portion of the protected segment when it overflows, resetting counters to half their value rather than zero. to preserve accumulated signal.

**H03 (smaller ghost window):** From cycle/0002/v003, reduce ghost_cap from prob_cap to prob_cap/2 to test whether a tighter ghost window (tracking only the most recently evicted keys) preserves most of the benefit at lower memory cost.

Actually for 3 hypotheses and wanting distinct strategies, let me think more carefully:

1. **Isolate ghost admission** — Build on cycle/0001/v001 (SLRU only) and add ghost admission. This directly answers: "what is the marginal value of ghost admission when combined with SLRU?"

2. **Better frequency demotion** — Build on cycle/0002/v003 (SLRU + ghost). Fix the signal design: persistent counters (not reset to 0 on demotion), wider scan window (scan entire protected LRU quarter rather than 4 nodes), evict the minimum-frequency node rather than LRU.

3. **Protected-segment CLOCK** — Build on cycle/0002/v003. Replace the doubly-linked list in the protected segment with a clock ring (one reference bit per node). When demoting from protected due to overflow, sweep the clock until a node with reference bit=0 is found (instead of always taking the strict LRU head). This gives a second-chance mechanism inside protected without the narrow-window limitation of the frequency scan. It's a completely different structural approach to the "which protected entry to demote" problem.

These three hypotheses are very distinct: H01 tests ghost admission in isolation on top of SLRU, H02 tests corrected frequency-aware demotion with persistent counters and a wider scan window, and H03 tests second-chance CLOCK demotion inside the protected segment as an alternative structural approach.

Both H02 and H03 cite the cross-cycle winner cycle/0002/v003, satisfying the constraint. Now I need to write these three hypotheses concisely and specifically.

**H01: Ghost admission on top of cycle-1 SLRU**

Claim: Adding a ghost admission set to the cycle-1 SLRU implementation will improve hit-rate by capturing cross-residency warmth that the vanilla two-segment structure discards.

Parent: cycle/0001/v001

Changes: I'm adding ghost tracking fields to the cache, evicting probationary items into a ghost queue, and promoting items back to protected-MRU if they're accessed again while in the ghost set.

**H02: Frequency-aware demotion with persistent counters on v003**

Claim: Persistent per-node access frequency counters that survive demotion and a scan of the least-recently-used 25% of the protected segment will improve hit-rate over cycle/0002/v003 by demoting genuinely infrequent entries rather than strictly LRU ones.

Parent: cycle/0002/v003

Adding frequency tracking... to `Node<K, V>` (initialized to 0 at insertion)
- In `get()`, increment `self.nodes[idx].freq` (saturating at u32::MAX) for both protected and probationary hits
- In `demote_prot_head()`, replace the current "evict `prot_head`" logic with a scan of the LRU quarter of protected: walk forward from `prot_head` for `prot_count / 4` nodes (minimum 1), pick the node with the lowest `freq`, unlink and demote it to probationary-MRU
- On demotion (instead of resetting freq), halve the counter: `self.nodes[demoted_idx].freq /= 2` — this preserves relative ordering but decays accumulated signal
- Keep all ghost admission logic intact; only the demotion-victim selection inside protected changes

Now I'm testing a CLOCK second-chance variant where protected entries get a reprieve before demotion. I'll add a referenced bit to each node, set it on protected hits without moving the node, then implement a CLOCK sweep in the demotion path that gives nodes with the referenced bit set another chance to stay in the protected segment.

The main implementation challenge is handling the linked list traversal correctly — I need to walk forward from the current clock hand position, wrapping around if necessary, and stop after checking at most `prot_count` nodes to avoid an infinite loop if all nodes are recently referenced.

I'm reconsidering whether the CLOCK approach is the right direction here. The linked list doesn't naturally support efficient wraparound, and the complexity might not be worth the marginal improvement. Let me think about a simpler alternative: instead of CLOCK, I could use a frequency-based victim selection within the protected segment's LRU quarter, which would be easier to implement and still give recently-accessed entries better protection.

Implementing frequency tracking...

I'm adding a `freq: u32` field to track access counts, incrementing it on every hit (both protected and probationary), and then during demotion I'll scan the LRU quarter of protected to find the lowest-frequency node as the victim. On demotion, I'll halve the frequency to decay the signal over time, so older accesses gradually lose their weight.

This approach is simpler than CLOCK and avoids the wraparound complexity while still prioritizing frequently-accessed entries for retention in the protected segment. I'm formalizing this as a hypothesis: adding a referenced bit to protected nodes lets recently-accessed entries get a second chance before eviction, which should boost hit-rate by preventing hot entries from being accidentally demoted just because they landed at the LRU head. The implementation marks nodes as referenced on access without moving them, then during demotion checks that bit to decide whether to move the node to the tail for another cycle or evict it outright.

But I'm realizing there's a fundamental problem with this approach: if protected-segment hits don't trigger any list movement, the ordering becomes stale and only new insertions push things to the tail, which breaks the recency tracking that the whole cache depends on. we check the head — if that head has been accessed, we give it a second chance. But nodes in the middle of protected that have been accessed recently but not touched since their original position won't benefit.

Actually this is similar to how CLOCK works in practice. The head of the list is the oldest (LRU) entry. If we don't move on get(), then position represents insertion/promotion order, not last-access order. On demotion overflow:
- Check if prot_head.referenced is true → if so, give second chance (clear bit, move to tail, check new head)
- Evict the first unreferenced node from the head

This is the classic CLOCK/second-chance algorithm. It's a valid design.

But wait — there's a subtle issue: if we stop moving nodes on get() in protected, this changes the behavior for nodes that are already in protected. Their position is now their "insertion age" rather than "last access age." This is a fundamental design change to the protected segment.

The question is whether second-chance is actually better than strict LRU for the protected segment. The hypothesis seems sound, but I need to be clear about what's changing. I could keep move-to-tail on get for protected and additionally set referenced = true when moving to tail, but then the reference bit would always be set when the node moves to tail and only cleared during second-chance eviction. That means second-chance would rarely trigger in a well-used cache since nodes at the tail wouldn't cycle back to become the head.

The cleaner approach for H03 is to set referenced = true in get() for protected hits without moving the node.

Implementing the demote logic... I'm checking whether the new prot_head also has its referenced bit set—if so, we'd only grant one second chance per demotion call, which is fine since it's bounded and prevents infinite loops. Now I'm crystallizing three distinct hypotheses: one isolates ghost admission without SLRU, while the other two build on the incumbent with either persistent frequency counters or a second-chance CLOCK mechanism in the protected segment, with both deriving from the cross-cycle winner as required.

Now I need to set up the output directory and write these files.

### 21:43:38 chat

I have all the context I need. Let me verify the output directory exists:

### 21:43:38 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/hypotheses 2>/dev/null && echo \"exists\" || echo \"not found\"",
  "description": "Check if output directory exists"
}
```

### 21:43:49 thinking

Good, the directory exists. Now let me write the 3 hypothesis files.

Let me recap the 3 hypotheses:

**H01: Ghost admission isolated on cycle-1 SLRU (cycle/0001/v001)**
- Parent: `cycle/0001/v001`
- Claim: Adding a ghost admission set to the cycle-1 SLRU variant will improve hit-rate by capturing cross-residency warmth that the vanilla two-segment structure discards.
- Changes: Add ghost fields, ghost logic in put() for eviction tracking and ghost-hit admission

**H02: Persistent-counter frequency-aware demotion on v003 (cycle/0002/v003)**


- Parent: `cycle/0002/v003`
- Claim: Persistent per-node frequency counters that halve rather than reset on demotion, combined with scanning the LRU quarter of protected on overflow, will improve hit-rate by demoting genuinely infrequent entries instead of strictly LRU ones.
- Changes: Add frequency tracking to nodes, increment on access, scan the LRU quarter during demotion, and halve frequency rather than reset it

**H03: Second-chance CLOCK in protected segment on v003 (cycle/0002/v003)**
- Parent: `cycle/0002/v003`
- Claim: Replacing strict LRU demotion in the protected segment with a second-chance mechanism that moves recently-accessed entries to the tail before evicting will improve hit-rate by protecting hot entries from premature eviction.
- Changes: Add a referenced flag to nodes, modify protected segment gets to set the flag without moving entries, and update demotion to grant one second chance before evicting

### 21:43:49 chat

Good. Now I'll write the three hypothesis files:

### 21:44:07 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/hypotheses/H01.md",
  "content": "# H01\n\nAdding a ghost admission set to the cycle-1 SLRU variant will improve hit-rate by capturing cross-residency warmth that the vanilla two-segment structure discards, isolating the marginal contribution of ghost admission from SLRU segmentation.\n\n## Parent\n\ncycle/0001/v001\n\n## Changes\n\ncycle/0001/v001 is a clean two-segment SLRU (80/20 split) with no ghost tracking. Apply the following additions to its `src/lru_cache/mod.rs`:\n\n1. **New imports** — add `use std::collections::{HashSet, VecDeque};` (HashMap is already present).\n\n2. **New struct fields** — add three fields to `LruCache<K, V>`:\n   ```\n   ghost: VecDeque<K>,\n   ghost_set: HashSet<K>,\n   ghost_cap: usize,\n   ```\n\n3. **`try_new`** — after computing `prob_cap`, set `ghost_cap = prob_cap`; initialize `ghost` and `ghost_set` with `with_capacity(ghost_cap + 1)`.\n\n4. **`put()` eviction path** — in the branch that evicts the probationary LRU (when `prob_count > 0` or `prob_head` is chosen), after calling `self.map.remove(&evicted_key)` and unlinking the node, call `self.add_to_ghost(evicted_key)` before reusing the slot. Protected evictions must NOT be recorded in the ghost set.\n\n5. **`put()` admission path** — after acquiring the arena slot and before linking into probationary, check `self.ghost_set.contains(&key)`. If true:\n   - Remove the key from `ghost_set` and `ghost` (`ghost.retain(|k| k != &key)`).\n   - Set the node's segment to `Segment::Protected` (or `in_protected = true`, matching v001's field name).\n   - Link into protected MRU (`push_back_prot(idx)`, `prot_count += 1`).\n   - Trigger overflow demotion if `prot_count > prot_cap`.\n   - Insert into `map` and return early — skip the probationary link.\n\n6. **New helper** `fn add_to_ghost(&mut self, key: K)`:\n   ```\n   if ghost_cap == 0 { return; }\n   if ghost.len() >= ghost_cap {\n       if let Some(oldest) = ghost.pop_front() { ghost_set.remove(&oldest); }\n   }\n   ghost_set.insert(key.clone());\n   ghost.push_back(key);\n   ```\n\nDo not change the 80/20 split, the demotion logic, or any other part of the cycle-1 SLRU structure. The goal is a minimal graft of ghost admission onto the existing design so the gain (or lack thereof) can be attributed cleanly to the ghost set alone.\n\n## Rationale\n\ncycle/0002/v003 bundles SLRU segmentation and ghost admission in one jump from the plain-LRU baseline, so its measured gain conflates both mechanisms. This variant applies ghost admission on top of the already-established SLRU winner (cycle/0001/v001), giving a direct read on the ghost set's marginal value. If hit-rate rises above cycle/0002/v003's 0.721, the ghost set contributes more than the v003 measurement implies; if it stays near cycle/0001/v001's 0.717, the bulk of v003's gain is attributable to implementation differences from the baseline rather than the ghost mechanism itself.\n"
}
```

### 21:44:25 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/hypotheses/H02.md",
  "content": "# H02\n\nPersistent per-node access frequency counters that survive demotion (halved rather than reset) and a scan of the LRU quarter of the protected segment will improve hit-rate over cycle/0002/v003 by demoting genuinely infrequent protected entries rather than strictly the oldest one.\n\n## Parent\n\ncycle/0002/v003\n\n## Changes\n\ncycle/0002/v003's frequency-aware demotion attempt (cycle/0002/v002) failed for two reasons: a 4-node scan window covering a negligible fraction of the protected segment, and counters reset to zero on every demotion. This variant fixes both in `src/lru_cache/mod.rs`:\n\n1. **Add `freq: u32` to `Node<K, V>`** — initialized to `0` whenever a slot is allocated or reused in `put()`.\n\n2. **Increment `freq` on every cache hit** — in `get()`, after resolving `idx` from the map, add `self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1)` before the segment-specific movement code. Apply this to both protected and probationary hits.\n\n3. **Replace `demote_prot_head()` with a minimum-frequency scan of the LRU quarter**:\n   - Compute `scan_len = (self.prot_count / 4).max(1)`.\n   - Walk forward from `prot_head` for `scan_len` nodes, collecting `(arena_index, freq)` pairs.\n   - Select the node with the lowest `freq` among the scanned nodes (tie-break: take the first encountered, i.e. the LRU among ties).\n   - Unlink that node from the protected list and demote it to probationary MRU; decrement `prot_count`, increment `prob_count`, update `segment` to `Segment::Probationary`.\n   - After demotion, **halve** the evicted node's counter: `self.nodes[victim_idx].freq /= 2`. This preserves relative ordering of accumulated signal across demotion cycles rather than wiping it.\n\n4. **Do not change any other logic** — ghost admission, the 80/20 split, probationary eviction into the ghost set, and all other paths remain identical to cycle/0002/v003.\n\nThe scan is bounded to `prot_count / 4` nodes at most, so the worst-case demotion cost is O(capacity/4) rather than O(1). This is acceptable for a hit-rate evaluation; throughput can be measured separately if this variant wins.\n\n## Rationale\n\ncycle/0002/v002's failure was diagnosed as a signal-design flaw, not evidence that frequency is uninformative inside the protected segment. A scan window of 4 nodes over a 1638-slot protected segment (at capacity=2048) observes less than 0.25% of candidates — effectively the same victim as strict LRU. The LRU quarter (about 410 nodes) is large enough to find genuine low-frequency entries without scanning the entire segment. Halving counters on demotion keeps durable signal alive: a node that accumulated freq=20 over two residency periods carries freq=10 into its next evaluation, clearly distinguishable from a newly promoted node with freq=0.\n"
}
```

### 21:44:44 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/hypotheses/H03.md",
  "content": "# H03\n\nReplacing strict LRU victim selection in the protected segment with a single second-chance mechanism — where a recently-accessed protected entry is moved to the tail and given one reprieve before demotion — will improve hit-rate over cycle/0002/v003 by shielding hot protected entries from premature eviction due to their LRU head position.\n\n## Parent\n\ncycle/0002/v003\n\n## Changes\n\nModify `src/lru_cache/mod.rs` to add a second-chance bit to the protected segment. All ghost admission and probationary logic remains identical to cycle/0002/v003:\n\n1. **Add `referenced: bool` to `Node<K, V>`** — initialized to `false` wherever nodes are created or reused (both the fresh-allocation path and the eviction-slot-reuse path in `put()`).\n\n2. **Change the protected-hit path in `get()`** — in the `Segment::Protected` branch, instead of `self.unlink_prot(idx); self.push_back_prot(idx);`, simply set `self.nodes[idx].referenced = true` and leave the node in place. The node's position in the list is NOT updated; its second-chance bit records that it was accessed.\n\n3. **Replace `demote_prot_head()` with a second-chance sweep**:\n   ```rust\n   fn demote_prot_head(&mut self) {\n       if let Some(h) = self.prot_head {\n           if self.nodes[h].referenced {\n               // Give one second chance: clear the bit, move to protected tail.\n               self.nodes[h].referenced = false;\n               self.unlink_prot(h);\n               self.push_back_prot(h);\n           }\n           // Demote the current prot_head (which now has referenced == false).\n           if let Some(h) = self.prot_head {\n               self.unlink_prot(h);\n               self.prot_count -= 1;\n               self.nodes[h].segment = Segment::Probationary;\n               self.push_back_prob(h);\n               self.prob_count += 1;\n           }\n       }\n   }\n   ```\n   This gives at most one second chance per demotion call, keeping the operation O(1) worst-case (two list pointer updates for the reprieve, then one for the actual demotion).\n\n4. **Probationary promotion path in `get()`** — when a probationary node is promoted into protected, initialize `referenced = false` on the node (it enters protected with a clean slate; it must earn its second chance through a subsequent access).\n\n5. **Ghost admission path in `put()`** — when a ghost-hit entry is admitted directly into protected, also initialize `referenced = false` on the node.\n\nDo not change the 80/20 split, the ghost set size or logic, or the probationary eviction path.\n\n## Rationale\n\nIn a working set where protected entries are accessed at varying rates, the strict LRU head is not always the best demotion victim: an entry that was promoted long ago but accessed again recently sits near the head because it has not been touched since that recent access moved it forward. The second-chance bit detects this case cheaply — the bit is set on every protected hit without any pointer movement — and gives the entry one cycle to age out of the LRU head position before it is demoted. This is structurally distinct from frequency counting (H02): it requires no arithmetic, no scan, and no persistent counter; it is a pure recency signal expressed as a single bit.\n"
}
```

### 21:44:54 chat

All three hypothesis files are written. Here's a summary of what was produced for cycle 0003:

---

**H01 — Ghost admission isolated on cycle-1 SLRU** (`cycle/0001/v001`)
Grafts a ghost admission set (ghost_cap = prob_cap) onto the cycle-1 SLRU winner without touching any other logic. This cleanly answers the open question from cycle 2: how much of cycle/0002/v003's 0.721 gain is attributable to ghost admission alone versus SLRU segmentation?

**H02 — Persistent-counter frequency demotion** (`cycle/0002/v003`)
Fixes the two diagnosed failures of cycle/0002/v002: (a) scan window raised from 4 nodes to the LRU quarter of protected (~410 nodes at capacity=2048), (b) counters halved on demotion rather than reset to zero so accumulated signal survives residency cycles. Demotes the minimum-frequency node found in the scan rather than the strict LRU head.

**H03 — Second-chance CLOCK demotion in protected** (`cycle/0002/v003`)
Adds a single `referenced: bool` bit to each node. Protected hits set the bit without moving the node. On protected overflow, `demote_prot_head` checks the bit: if set, clear it and move the node to the protected tail (one reprieve), then demote the new head. This is O(1), requires no counter arithmetic, and is structurally distinct from both strict LRU and frequency scanning.

Both H02 and H03 derive from the cross-cycle incumbent (`cycle/0002/v003`), satisfying the mandatory parent constraint.


---

- finished: 2026-06-04T21:44:55+00:00
- duration: 192.99s
- status: ok
- input_tokens: 10
- output_tokens: 9902
- cache_read_tokens: 259273
- cache_write_tokens: 32116
- cost_usd: 0.348191
