# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0004\hypotheses
- allow_edits: True
- started: 2026-07-19T17:21:14+00:00

## timeline

### 17:21:18 thinking

Let me start by reading all the relevant context files to understand the current state of the cache evolution research and what hypotheses have been tried before.

### 17:21:18 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0003\\cycle-insight.md"
}
```

### 17:21:19 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0003\\cycle-winners.json"
}
```

### 17:21:19 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\leaderboard.md"
}
```

### 17:21:19 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cross-cycle-insights.md"
}
```

### 17:21:23 thinking

Now let me read the variant files from cycle 0003 and the baseline to understand the implementation details.

### 17:21:24 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0003\\variants\\*"
}
```

### 17:21:25 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles-ledger.jsonl"
}
```

### 17:21:27 thinking

Now let me read the variant implementations from cycle 0003 and the baseline to understand what these variants look like at the code level.

### 17:21:29 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0003\\variants\\v001\\attempts\\01\\overlay\\src\\lru_cache\\mod.rs"
}
```

### 17:21:29 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0003\\variants\\v002\\attempts\\01\\overlay\\src\\lru_cache\\mod.rs"
}
```

### 17:21:30 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0003\\variants\\v001\\insight.md"
}
```

### 17:21:30 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0003\\variants\\v002\\insight.md"
}
```

### 17:21:31 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0003\\variants\\v003\\insight.md"
}
```

### 17:21:34 thinking

Now let me read the baseline and the best winner (cycle/0002/v001) to understand the current best implementation:

### 17:21:34 chat

Now let me read the best-performing winner (cycle/0002/v001) and the baseline to understand the foundation:

### 17:21:36 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0002\\variants\\v001\\attempts\\*\\overlay\\src\\lru_cache\\mod.rs"
}
```

### 17:21:36 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline\\src\\lru_cache\\*"
}
```

### 17:21:39 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0002\\**\\mod.rs"
}
```

### 17:21:39 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline\\**\\*"
}
```

### 17:21:42 thinking

Interesting. The baseline directory has evaluation files but I need to find the source code. Let me look for the baseline source more broadly.

### 17:21:43 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\**\\mod.rs"
}
```

### 17:21:43 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0002\\variants\\v001\\attempts\\01\\overlay\\src\\lru_cache\\mod.rs"
}
```

### 17:24:23 thinking

Now I have a thorough understanding of the codebase and history. Let me synthesize what I know before crafting the 3 hypotheses for cycle 0004:

**Cross-cycle winners:**
- `cycle/0002/v001` (score: 0.35, hit-rate: 0.7202, metadata-size: 24, per-op-time: 0) — the 2Q policy with prob_cap = capacity/4
- `cycle/0001/v002` (score: 0.3, hit-rate: 0.6652, metadata-size: 24, per-op-time: 0) — the u32 sentinel LRU
- `cycle/0002/v002` (score: 0.2, hit-rate: 0.6652, metadata-size: 48, per-op-time: 0) — the MRU-tail guard on standard LRU

The 2Q approach with spare-bit encoding achieved the best hit-rate at 0.7202 without increasing node size, while cycle 0003 saw no improvements due to MRU-tail guard overhead and per-op-time regressions, though one variant did reach 0.7291 hit-rate.

The main unexplored avenue is adaptive quota sizing to dynamically balance the protected and probationary regions, along with alternatives like lazy demotion strategies and smarter admission filtering on the probationary side. I need to keep node size at 24 bytes and avoid per-op-time regressions while steering clear of the MRU-tail guard approach.

The cycle directive is clear: focus on structural changes rather than composition, with adaptive quota sizing as the top priority. The mechanism would track demotion frequency and adjust the protected/probationary boundary dynamically instead of using a fixed ratio.

For implementation, I'd add a demotion counter to track how often items get demoted. If the demotion rate exceeds a threshold, shrink the probationary capacity to give more space to protected items; if demotions drop off, relax it back toward the baseline. This way we get the hit-rate benefits of a larger protected budget during high-traffic periods without permanently paying the cost of frequent unlink/relink operations.

The key insight is that the metadata-size metric measures the Node struct, not the LruCache struct itself, so there's flexibility to add tracking fields to the cache without affecting the measured overhead.

Now looking at a different optimization: instead of always promoting a protected-queue hit to the MRU tail (which requires an expensive unlink and push), I could defer that promotion and only move nodes when necessary.

The idea is to use a generation counter to track staleness—nodes only get relinked when their generation is old enough that reordering would actually affect eviction decisions. A simpler variant: skip promotion entirely if the node is already in the "recent" half of the protected queue, since it's unlikely to be evicted soon anyway. This trades some precision in LRU ordering for a significant reduction in pointer operations on protected hits.

The challenge is checking whether a node is in the top half without traversing the queue. A generation counter approach—adding a generation field to track when a node was last promoted—would let me make that decision in O(1) time. On a protected hit, I'd set a referenced bit instead of doing the expensive unlink and push_back operation. Then during demotion pressure, I'd scan from the head of the protected queue—nodes with the referenced bit get it cleared and stay, while unreferenced nodes get demoted to probationary. This is essentially CLOCK within the protected queue, but only scanning when needed rather than on every access, which avoids the cache-line churn of constant relinking. within protected, the hand only sweeps during demotion (which is rare at steady state), so cache-line pollution should be bounded compared to pure CLOCK. I need to reconsider the cross-cycle insights about CLOCK's pollution issues—they stem from eager hand sweeps on every access, but here that only happens when protected overflows.

There's a practical constraint though: using bit 31 of the `next` pointer to track the referenced bit conflicts with the current NULL sentinel, so I'd need to reassign NULL to a different value to make room for the referenced flag. Instead of modifying the node layout, I could add a separate bitmap indexed by slot to track which nodes are referenced—this would keep the Node structure unchanged while using a bit of extra memory, which is acceptable since it's only the LruCache struct that grows, not each individual node.

The approach would be to add a `ref_bits` vector to track referenced status, and when a protected node gets a hit, I'd just set its bit instead of doing the expensive unlink-and-push operation. During protected overflow, I'd scan from the head to find the first unreferenced node to demote, clearing bits as I go. The lazy-sweep approach avoids cache-line pollution from eager hand sweeps, and the scan during demotion only triggers on graduation events, not every access—so the cost is bounded. On typical workloads with dense referenced bits, the expected scan length is short. Now I'm settling on three concrete hypotheses to test: adaptive protected quota that adjusts dynamically based on demotion rate to improve hit-rate, plus the lazy-sweep variant with second-chance promotion for protected nodes. 64-bit field with 32 bits for each pointer. But looking at the actual node layout, we're already at 24 bytes with u64 key and value plus two u32 pointers, and the baseline metadata shows 48 bytes total — so the cycle/0001/v002 optimization already compressed the pointers down from usize to u32, getting us to that 24-byte node size.

To go smaller, I'd need to either abandon the explicit linked list structure entirely or use u16 pointers instead, which would limit capacity to 65,535 entries — more than enough for the 2048-entry test case. That would shrink the node to 20 bytes total.

The alignment issue is tricky though. Even with u16 links, the struct still pads to 24 bytes because the u64 fields force 8-byte alignment. I could try using `#[repr(packed)]` to eliminate padding, but that introduces unaligned memory access which has its own performance costs.

A singly-linked list would save the `prev` field, but then unlinking a node from the middle during promotion becomes O(n) since I'd need to traverse from the head to find the predecessor. That's worse than the current doubly-linked approach. For eviction from the head it's fine, but the general case breaks down.

Let me explore other node layout options entirely. without re-hashing the key on removal. The `raw_entry` API in hashbrown would let us use a precomputed hash directly, but that's unstable in std and would require adding hashbrown as a dependency.

Since I can't modify Cargo.toml due to the gate-integrity constraint, I'm exploring a frequency-based admission filter instead. The idea is to use a lightweight bloom filter variant—just a bit vector indexed by `key % (capacity * 8)`—to track which keys have been accessed before. This way, only keys seen at least twice get admitted to the cache, similar to TinyLFU's approach. It adds minimal overhead (just capacity bytes) and should help with scan-heavy workloads.

The risk is that this "filter and replace on miss" pattern could hurt hit-rate if the trace has many one-time accesses, though for Zipfian distributions where hot items are accessed repeatedly, it should reduce cache pollution. The tradeoff is that hot keys miss on their first access before being inserted on the second, which might actually hurt performance depending on the workload characteristics.

For the deferred promotion approach, I'd wait for a second probationary hit before graduating to protected, similar to ARC's ghost list concept. This requires adding a second-hit bit to track whether a probationary item has been accessed once already. I could reuse bit 30 of the `prev` pointer since bit 31 is already used for the protection flag, though this changes how probationary hits are handled semantically.

With this scheme, valid indices would be limited to 30 bits, supporting up to roughly a billion entries—plenty for the 2048-entry cache. I'm defining the bit layout more precisely: bit 31 for the protection flag, bit 30 for the seen flag on probationary nodes, and bits 0-29 for the actual index or NULL sentinel. This means updating the NULL constant and adding new bit masks to the module.

In the promotion logic, I need to check whether a probationary hit has already been seen. If not, I'll mark it as seen and return without graduating; if it has been seen, I'll clear the flag and promote normally. The rest of the cache logic stays unchanged. This enforces a two-hit requirement for probationary items before they gain protection, which should filter out burst accesses that won't repeat—a meaningful policy shift that could improve hit-rate on Zipfian workloads by avoiding premature promotion of scan-like patterns.

There's a risk here: if the hot set in the trace rarely accumulates two consecutive probationary hits before eviction, this could actually hurt performance. But with 2048 capacity and a reasonably sized hot set, items should survive long enough to get those two hits. This feels like a clean, distinct hypothesis worth testing against the baseline.

Now I'm settling on three final hypotheses. H01 uses adaptive protected quota sizing based on demotion pressure, and H02 applies a CLOCK-within-protected strategy with a referenced bitmap to improve per-op time. Both cite cycle/0002/v001 as their parent, satisfying the cross-cycle requirement. H03 is the two-hit probationary filter I just outlined, which cites the baseline parent. Let me verify they're all distinct: H01 changes quota dynamically, H02 optimizes the protected region's scan behavior, and H03 filters admission into protection itself—three separate mechanisms.

Now I'm writing out the specific implementation changes for each hypothesis, starting with H01's struct modifications...

I'm adding a method that periodically adjusts the probationary capacity based on demotion pressure — shrinking it when demotions are high to protect more entries, and growing it when there's no pressure to reclaim space.

In the `promote()` method, when an entry gets demoted back to probationary, I increment the demotion counter to track this pressure.

Now I'm introducing a bit-vector to track which slots have been referenced since the last demotion check, growing it alongside the node arena to keep memory overhead minimal.

I'm adding helper methods to set and test-clear individual bits in this vector, then modifying the protected-hit path in `promote()` to just mark the reference bit and return early instead of moving the node.

When the protected budget overflows, I'm scanning through the protected list from the head, giving each referenced node a second chance by moving it to the back of the protected queue, then demoting the first unreferenced victim I find. Completing the demotion logic...

The loop could scan through the entire protected queue if everything's referenced, so I need to cap it with a max-scan limit like 4 nodes to keep the per-demotion cost bounded. If we hit the limit, fall back to just demoting the head.

Now looking at the baseline implementation — it's the plain LRU from the original source with 48-byte nodes according to the fitness metrics, not the earlier versions. I need to locate and examine the baseline source file to understand the original Node structure and how it's been modified across the versions.

### 17:24:23 chat

Let me check the baseline source and the cycle/0001/v002 source to understand the structural foundations:

### 17:24:23 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\src\\lru_cache\\mod.rs"
}
```

### 17:24:23 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline\\insight.md"
}
```

### 17:27:31 thinking

Now I have a complete picture. Let me also quickly check the cycle/0002/v002 (the MRU-tail guard) source since it's also a winner but distinct from v001:

Let me review what I know and finalize the 3 hypotheses:

**Baseline**: Plain LRU using `Option<usize>` links (Node = K + V + Option<usize> + Option<usize> = K + V + 16 bytes for usize-ptr targets = 24+16 = 40 bytes? No: for K=V=u64, Node = 8+8+16+16 bytes... 

Wait, I need to reconsider the size of `Option<usize>`. Due to niche optimization on 64-bit systems, `Option<usize>` is actually 8 bytes, not 16, since None can be represented as usize::MAX. So for Node<u64,u64> that would be 8+8+8+8 = 32 bytes, but the measured metadata size is 48 bytes, so something's off with my calculation. Right, so `Option<usize>` is actually 16 bytes on 64-bit systems since `usize` has no niche—every bit pattern is valid—so the discriminant needs its own 8 bytes due to alignment. That means my struct would be 24 + 16 + 8 = 48 bytes, which matches the measured size.

The optimization in cycle/0001/v002 swaps `Option<usize>` for a `u32` with a sentinel value, compressing each link from 16 bytes down to 4 bytes and cutting the node size from 48 to 24 bytes. Now I'm deciding whether to build H03 on top of the baseline's 48-byte structure or start from the more efficient 24-byte version.

If I parent from baseline, I'd either accept the larger node size and focus purely on hit-rate improvements, or I'd need to also incorporate the node layout optimization as part of the hypothesis. Looking at the cross-cycle winners, cycle/0001/v002 already demonstrates that metadata-size matters for fitness, so starting from baseline without addressing node size would put me at a disadvantage. I'm leaning toward parenting from a more optimized ancestor instead. baseline instead, which is also valid. So I have flexibility here: H03 could either start from the baseline and propose node compaction plus a policy change, or cite `cycle/0001/v002` and layer the two-hit filter on top of its plain LRU structure. Both approaches are distinct from H01 and H02, which build on the 2Q variant.

The three hypotheses I'm settling on are: H01 adapting the protected quota dynamically to address the top open question from cycle-0003, H02 using a CLOCK-within-protected structure with lazy promotion to fix the per-operation regression, and H03 implementing a two-hit probationary filter that encodes state in a spare bit and requires two hits before graduating entries—a fundamentally different policy approach that targets hit rate improvements.

But I'm realizing H03 has a structural problem: the parent `cycle/0001/v002` is a plain compact LRU without any probationary/protected distinction, so a two-hit filter doesn't map cleanly onto it. In a single-queue LRU, every access already moves the item to MRU, so there's no meaningful way to defer promotion. The two-hit idea only makes sense with a multi-queue structure like 2Q or ARC where you can actually hold items in probationary before promoting them to protected.

So maybe H03 should instead parent the baseline and propose ARC—Adaptive Replacement Cache—which maintains ghost lists of recently evicted items and dynamically adjusts the boundary between protected and probationary regions. That's a fundamentally different approach from what H01 and H02 are exploring.

The tradeoff is that ARC needs four lists plus ghost tracking, which adds complexity and memory overhead for storing evicted keys, though I could implement ghost entries as special values in the HashMap without growing the Node struct itself. But this is getting complicated, so I should focus on what's most likely to actually win.

Looking at the cycle results, the best hit-rate achieved is 0.7202 with 2Q, though one variant hit 0.7291 but suffered per-op-time regression. The real question is whether adaptive quota can get that hit-rate improvement without the performance cost, or if I should try something like skipping relinking when a protected item is already near the MRU end.

For H03, I'm thinking about a probabilistic approach to promoting items from the probationary queue instead of promoting every hit.

Actually, let me reconsider—instead of probabilistic promotion, I could segment the probationary FIFO into hot and cold tiers, since items at the MRU end of probationary are behaving differently than those at the LRU end. Okay, so all three hypotheses are actually drawing from the cross-cycle winner, which satisfies the requirement. But for better exploration breadth, I should probably have at least one rooted in the baseline instead. Let me restructure with H01 anchored differently.

Actually, looking at what I'm setting up: H01 uses adaptive protected quota from the proven winner, H02 applies CLOCK logic within the protected region, and H03 starts fresh from baseline with a two-hit admission filter combined with node compaction. For H03, the key changes are shrinking node size by replacing pointer links with u32 sentinels, layering in the 2Q structure with protected and probationary queues using bit flags, and adding a SEEN_BIT requirement so entries need two hits before graduating out of probation. The novel claim centers on whether the two-hit filter drives the improvement.

Wait, that's actually too many simultaneous changes. Let me narrow H03 to just one mechanism: reducing node size with u16 links instead of u32. The baseline node is 24 bytes with 8-byte key, 8-byte value, and two 4-byte pointers. Switching to u16 links would theoretically drop it to 20 bytes, but struct alignment pads it back to 24 anyway, so there's no actual gain there.

I could try packing both links into a single u64, but that still leaves us at 24 bytes. Embedding the LRU ordering directly in the HashMap instead of maintaining separate prev/next pointers won't work because O(1) operations require immediate access to both neighbors. The real constraint is that the intrusive list needs bidirectional traversal.

I've explored various struct layouts with different field orderings and link sizes, but they all bottom out at 24 bytes for the K=V=u64 case due to alignment requirements. Node compaction isn't going to yield meaningful improvements here, so I should pivot to a fundamentally different approach — maybe adding a frequency filter to the baseline LRU algorithm.

The idea would be a bloom filter to detect second-time accesses, routing new keys through an admission queue instead of letting them immediately compete with hot items. But this requires both the u32 link compaction from the earlier optimization and the new filtering logic, which feels like stacking two separate changes. I'm going to accept that H03 will need to build on the compacted structure rather than starting fresh from baseline.

So all three hypotheses will use cycle/0002/v001 as their parent, each exploring a different optimization angle. H01 adjusts the protected quota dynamically based on demotion pressure, H02 uses a referenced bitmap for lazy promotion within the protected segment, and H03 implements a two-hit filter by repurposing a spare bit in the node structure. Let me detail each approach.

For H01, I'm adding operation and demotion counters to track how often items get demoted from the protected region. At regular intervals tied to cache capacity, I'll sample the demotion rate and adjust the protected quota upward if pressure is high, or downward if it's low. This keeps the protected region sized appropriately without requiring expensive per-node metadata changes.

Now moving to H02, which explores using a bitmap-based CLOCK algorithm within the protected region instead of unconditional unlink-and-push operations.

Adding a `ref_bits` vector as a compact bit array indexed by arena slot to track referenced entries. This grows alongside the node arena, and I'm implementing helper methods to set and check reference bits efficiently.

In the protected-hit path, I'll skip relinking entirely and just mark the reference bit instead. During demotion sweeps, I'll scan for unreferenced protected entries to evict, implementing the second-chance algorithm. Also need to clear the `ref_bits` for any slots that get evicted in `evict_one()`. I'm introducing a two-hit probationary filter using a spare bit in the `prev` field to track whether an item has been seen once, so that one-time accesses don't prematurely graduate to the protected queue and displace genuinely hot items. This requires adjusting the `IDX_MASK` to account for both the protection bit and the new seen bit.

Implementing the probationary graduation logic... Writing test cases... I see the issue now—if I just set the SEEN_BIT without relinking the node, it stays at its current position in the probationary list, which means it could be evicted before getting a second hit. I need to relink it to the MRU end of probationary on the first hit to give it a fair chance, then graduate it to protected on the second hit.

Writing the probationary hit logic... most `0x3FFF_FFFE`, so setting SEEN_BIT (bit 30) won't interfere with PROT_BIT (bit 31) or the index bits. When `unlink()` reads the `prev` field, it correctly masks out SEEN_BIT to get the predecessor index and checks bit 31 separately for the protected flag, so the bit layout stays clean.

When `prob_tail` is used as a predecessor index in `push_back_prob`, it's just a raw index with bits 30 and 31 clear, so assigning it and then setting SEEN_BIT works fine. But there's a problem in `unlink()`: when updating the successor's `prev` field with `succ_prot_bit | prev_idx`, it only preserves bit 31 (the protection flag) and loses bit 30 (SEEN_BIT) from the successor, which shouldn't happen during unlinking.

Fixing this...

The mask needs to capture both the PROT_BIT and SEEN_BIT from the successor's prev field, then combine those flags with the new predecessor index. This ensures we don't lose the successor's state bits during the relinking operation.

There's another consideration though: when a node gets demoted from protected to probationary status in the `promote()` function, we explicitly want to clear its SEEN_BIT as part of that demotion. The node gets unlinked from the protected list and pushed to the back of the probationary list, and that transition should reset the seen status.

For eviction, the flow is similar—the node gets unlinked and then reused, with its prev pointer reset to just the raw predecessor index, naturally clearing SEEN_BIT in the process.

When pushing a node to the protected list, the prev pointer is set with PROT_BIT and the raw tail index, which correctly sets PROT_BIT while leaving SEEN_BIT at 0.

So SEEN_BIT only needs special handling in two places: when promoting a node and when unlinking a node to preserve the successor's flags. Everything else naturally keeps SEEN_BIT at 0, which is the desired behavior.

Let me think through whether this SEEN_BIT approach is actually the right solution here. So NULL should be 0x3FFF_FFFF, which has the index bits all set to 1s (making it the first non-allocatable index) while keeping both SEEN_BIT and PROT_BIT clear at bits 30-31. that's never a valid slot index. With our capacity limit, slot indices range from 0 up to the maximum valid index, and NULL = 0x3FFF_FFFF sits just beyond that boundary. ✓

The original NULL value had bit 31 clear to distinguish from PROT_BIT, but now I need both bits 30 and 31 clear to distinguish from SEEN_BIT and PROT_BIT — and 0x3FFF_FFFF satisfies that requirement.

There's a potential issue in the `unlink()` function when checking if a node is protected and updating `prot_tail` with the masked previous index. PROT_BIT set). Then `prev_idx = 0x3FFF_FFFF & 0x3FFF_FFFF = NULL`. ✓

In `push_back_prob`, when adding a node to the probationary tail, I'm setting its `prev` field to the current tail index (which is a raw u32 with no flag bits). If it's the first item, `prev` becomes NULL; if it's a later item, `prev` gets the actual index. Now I need to trace through what happens when we later unlink such a node.

When we `|= SEEN_BIT` to mark a first-hit, the index gets the flag bit set, but masking it back with `IDX_MASK` recovers the original index correctly. The key insight is that `prob_tail` and `prot_tail` are always stored as clean indices without any flag bits, so when we copy them into the `prev` field, we're working with raw slot indices that can be masked properly later.

For the `unlink` fix, I need to preserve both the `PROT_BIT` and `SEEN_BIT` when updating the successor's `prev` field, so the logic extracts both flags before reassigning the predecessor index.

Now I'm ready to write out the three hypothesis files to the output directory, starting with H01 which covers the dynamic quota adjustment approach.

### 17:27:32 chat

Now I have a thorough understanding of the entire history. Let me write the three hypotheses:

### 17:27:45 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0004\\hypotheses\\H01.md",
  "content": "# H01\n\nDynamically adjusting the protected-queue quota in response to observed demotion pressure will capture the hit-rate benefit of a larger protected budget without permanently inflating the per-op-time cost seen in cycle/0003/v002.\n\n## Parent\n\ncycle/0002/v001\n\n## Changes\n\nAdd two counter fields to `LruCache` (no change to `Node<K,V>`):\n\n```rust\ndemotion_count: u32,  // demotions from protected → probationary since last adaptation\nop_count: u32,        // get + put calls since last adaptation\n```\n\nInitialise both to `0` in `try_new`.\n\nAdd a new `#[inline] fn maybe_adapt(&mut self)` method called at the **top** of both `get()` and `put()` (before any other logic). The method increments `self.op_count` and, when `op_count` reaches `self.capacity.max(512) as u32`, evaluates the adaptation:\n\n- If `demotion_count > (self.capacity / 16).max(1) as u32`:\n  protected is under-provisioned → shrink `prob_cap` by 1, clamped to a floor of `(self.capacity / 8).max(1)`.\n- Else if `demotion_count == 0`:\n  no demotion pressure → grow `prob_cap` by 1, clamped to a ceiling of `(self.capacity / 4).max(1)`.\n- Reset `self.op_count = 0; self.demotion_count = 0`.\n\nIn `promote()`, in the probationary-graduation branch, **after** the demotion block fires (`prot_len > prot_cap`), increment `self.demotion_count`.\n\nAll invariants, list structure, and the `Node<K,V>` layout (24 bytes, unchanged) must be preserved. The adaptation logic should be a pure side effect on the two counter fields and `prob_cap`; it must not touch `nodes`, the linked lists, or `map`.\n\n## Rationale\n\ncycle/0003/v002 achieved the highest hit-rate ever recorded (0.729) by fixing `prob_cap` at `capacity / 8`, but this permanently enlarged the protected path, raising per-op-time. The global winner cycle/0002/v001 uses `prob_cap = capacity / 4` and sits exactly at per-op-time = 0. Adaptive sizing should track the working-set size and drift the quota toward the most favourable point for the current phase, recovering hit-rate when demotion pressure is high without paying the steady-state cost of a permanently small `prob_cap`.\n"
}
```

### 17:28:06 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0004\\hypotheses\\H02.md",
  "content": "# H02\n\nReplacing the unconditional unlink-and-relink on every protected hit with a lazy referenced-bit and a bounded CLOCK-style sweep during demotion will reduce per-op-time on protected-heavy paths while preserving or improving hit-rate through second-chance semantics.\n\n## Parent\n\ncycle/0002/v001\n\n## Changes\n\n**Step 1 — add a compact referenced bitmap to `LruCache`** (does not affect `Node<K,V>`):\n\n```rust\nref_bits: Vec<u8>,  // bit i covers arena slot i; bit = 1 means \"referenced since last sweep\"\n```\n\nInitialise to `Vec::new()` in `try_new`. Grow in sync with the arena: in the `put()` path that allocates a new node (the `else` branch at `let new_idx = self.nodes.len()`), after `self.nodes.push(...)`, extend `ref_bits` to cover the new slot:\n\n```rust\nif self.nodes.len() > self.ref_bits.len() * 8 {\n    self.ref_bits.push(0u8);\n}\n```\n\nAdd three private helpers:\n\n```rust\n#[inline] fn set_ref(&mut self, idx: usize)   { self.ref_bits[idx >> 3] |= 1 << (idx & 7); }\n#[inline] fn clear_ref(&mut self, idx: usize) { self.ref_bits[idx >> 3] &= !(1u8 << (idx & 7)); }\n#[inline] fn test_ref(&self,  idx: usize) -> bool {\n    (self.ref_bits[idx >> 3] >> (idx & 7)) & 1 != 0\n}\n```\n\n**Step 2 — lazy promotion on protected hits.**\n\nIn `promote()`, in the `if protected { … }` branch, replace:\n\n```rust\nself.unlink(idx);\nself.push_back_prot(idx);\n```\n\nwith:\n\n```rust\nself.set_ref(idx);\nreturn;\n```\n\nThe node stays at its current position in the protected queue; only a bit flip is written.\n\n**Step 3 — bounded CLOCK sweep during demotion.**\n\nIn `promote()`, in the probationary-graduation branch, replace the single-step demotion of `prot_head` with a bounded scan (limit = 8 iterations). Replace:\n\n```rust\nlet h = (self.prot_head & IDX_MASK) as usize;\nself.unlink(h);\nself.prot_len -= 1;\nself.push_back_prob(h);\nself.prob_len += 1;\n```\n\nwith:\n\n```rust\nlet mut scan = 8usize;\nloop {\n    let h = (self.prot_head & IDX_MASK) as usize;\n    if scan == 0 || !self.test_ref(h) {\n        // First unreferenced node (or scan budget exhausted): demote it.\n        self.clear_ref(h);\n        self.unlink(h);\n        self.prot_len -= 1;\n        self.push_back_prob(h);\n        self.prob_len += 1;\n        break;\n    }\n    // Referenced: give it a second chance by moving to the MRU end of protected.\n    self.clear_ref(h);\n    self.unlink(h);\n    self.push_back_prot(h);\n    scan -= 1;\n}\n```\n\n**Step 4 — clear referenced bits on eviction.**\n\nIn `evict_one()`, after computing `h` in each branch, call `self.clear_ref(h)` before `self.unlink(h)` (defensive; the bit should already be clear for probationary nodes, but protected nodes evicted via the fallback path may still have it set).\n\nNo changes to `Node<K,V>`, `IDX_MASK`, `NULL`, `PROT_BIT`, or the probationary-path logic.\n\n## Rationale\n\nThe cycle-0003 cycle insight identified that the MRU-tail guard fails because it fires only for the exact tail node. This variant avoids the same trap: by recording \"was accessed\" with a single bit-or (no pointer chasing) and only doing physical relinking during the rare demotion event, the per-operation cost on protected hits drops from two pointer writes to one bit-or. The bounded CLOCK sweep (max 8 iterations, each moving a referenced node to MRU) additionally grants a second eviction chance to recently accessed protected items, which can lift hit-rate on Zipfian traces where a small hot set is accessed far more often than the tail. The scan cost is amortised: it only runs when a probationary hit forces a demotion, not on every get.\n"
}
```

### 17:28:40 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0004\\hypotheses\\H03.md",
  "content": "# H03\n\nRequiring two separate probationary accesses before graduating an item to the protected queue will improve hit-rate by preventing one-hit wonders from briefly occupying protected slots and displacing genuinely recurring items.\n\n## Parent\n\ncycle/0002/v001\n\n## Changes\n\n**Step 1 — add a SEEN_BIT constant and shrink IDX_MASK.**\n\nEncode a \"seen-once\" flag in bit 30 of the `prev` field alongside the existing `PROT_BIT` (bit 31). Valid slot indices therefore fit in bits 0–29.\n\nReplace the existing constants:\n\n```rust\n// Old\nconst NULL:     u32 = 0x7FFF_FFFF;\nconst IDX_MASK: u32 = 0x7FFF_FFFF;\n\n// New\nconst NULL:     u32 = 0x3FFF_FFFF;   // bits 30-31 both 0; acts as \"no predecessor\" sentinel\nconst SEEN_BIT: u32 = 0x4000_0000;   // bit 30: probationary node has been accessed once\nconst IDX_MASK: u32 = 0x3FFF_FFFF;   // strip PROT_BIT and SEEN_BIT to get raw index\n```\n\nUpdate the capacity guard in `try_new` to reflect the reduced index range:\n\n```rust\nif capacity > 0x3FFF_FFFFusize - 1 {   // max valid index = 0x3FFF_FFFE\n```\n\nFor the test capacity of 2048 this is trivially satisfied.\n\n**Step 2 — preserve SEEN_BIT of a node's successor in `unlink()`.**\n\nThe current `unlink()` preserves only `PROT_BIT` when updating the successor's prev field. Change the successor update to preserve both flag bits:\n\n```rust\n// Old\nlet succ_prot_bit = self.nodes[next as usize].prev & PROT_BIT;\nself.nodes[next as usize].prev = succ_prot_bit | prev_idx;\n\n// New\nlet succ_flags = self.nodes[next as usize].prev & (PROT_BIT | SEEN_BIT);\nself.nodes[next as usize].prev = succ_flags | prev_idx;\n```\n\nAll other logic in `unlink()` is unchanged; `IDX_MASK` now correctly strips both flag bits wherever it is used.\n\n**Step 3 — implement two-hit graduation in `promote()`.**\n\nIn the probationary-hit branch of `promote()` (the `else` arm, when `protected` is false), replace the immediate graduation with a two-phase check:\n\n```rust\n} else {\n    // Probationary hit.\n    let has_seen = (self.nodes[idx].prev & SEEN_BIT) != 0;\n    if !has_seen {\n        // First hit: refresh position in probationary and mark as seen.\n        // Move to probationary MRU to extend the window before eviction.\n        self.unlink(idx);\n        self.push_back_prob(idx);              // clears SEEN_BIT and PROT_BIT (sets plain index)\n        self.nodes[idx].prev |= SEEN_BIT;      // re-apply SEEN_BIT after push_back_prob\n        // prob_len and prot_len are unchanged; no graduation occurs.\n        return;\n    }\n    // Second+ hit: graduate to protected (SEEN_BIT is cleared automatically\n    // by unlink → NULL, then push_back_prot → PROT_BIT | prot_tail).\n    self.unlink(idx);\n    self.prob_len -= 1;\n    self.push_back_prot(idx);\n    self.prot_len += 1;\n    // Existing demotion enforcement block is unchanged.\n    let prot_cap = self.capacity - self.prob_cap;\n    if self.prot_len > prot_cap {\n        let h = (self.prot_head & IDX_MASK) as usize;\n        self.unlink(h);\n        self.prot_len -= 1;\n        self.push_back_prob(h);\n        self.prob_len += 1;\n    }\n}\n```\n\n**Step 4 — verify SEEN_BIT lifecycle.**\n\n- `push_back_prob(idx)`: assigns `self.nodes[idx].prev = self.prob_tail` (a raw index with bits 30–31 = 0). SEEN_BIT is naturally cleared. ✓\n- `push_back_prot(idx)`: assigns `self.nodes[idx].prev = PROT_BIT | self.prot_tail`. SEEN_BIT is naturally cleared. ✓\n- Eviction in `evict_one()` → `unlink(h)` clears prev to NULL, then `put()` calls `push_back_prob` on the reused slot — SEEN_BIT is cleared. ✓\n- Demotion from protected to probationary in `promote()` → same path as above. ✓\n\nNo changes to `push_back_prob`, `push_back_prot`, `evict_one`, or the `put()` fast path. `Node<K,V>` layout is unchanged at 24 bytes.\n\n**Step 5 — add a unit test for the two-hit contract.**\n\n```rust\n#[test]\nfn two_hit_filter_defers_graduation() {\n    // capacity=3: prob_cap=1 (capacity/4=0, max(1)=1), prot_cap=2.\n    let mut cache = LruCache::new(3);\n    cache.put(1, 10); // slot 0 → prob\n    cache.put(2, 20); // slot 1 → prob\n    cache.put(3, 30); // slot 2 → prob\n\n    // First get on key 1: sets SEEN_BIT, refreshes to prob MRU. Still in prob.\n    cache.get(&1);\n    // Key 1 should be alive (not evicted) and still accessible.\n    assert_eq!(*cache.get(&1).unwrap(), 10); // this is the second hit → graduates key 1 to prot\n    // Key 1 is now protected; keys 2 and 3 remain in prob.\n    // A new put fills from prob (evicts key 2 or 3, not key 1).\n    cache.put(4, 40);\n    assert_eq!(*cache.get(&1).unwrap(), 10); // key 1 protected, must survive\n}\n```\n\n## Rationale\n\ncycle/0002/v001 graduates any probationary hit to protected immediately. On a Zipfian workload the hot items are accessed far more than once and will quickly re-enter protected after any eviction from it. However, items that happen to be accessed twice in a short burst (e.g., a scan that repeats a key before moving on) also graduate, temporarily displacing true recurring items. Requiring a second, independent probationary hit filters out these burst artefacts with minimal cost: the first hit only does unlink + push_back_prob (cheaper than the graduation path), leaving SEEN_BIT as the persistent signal. The node layout is unchanged (SEEN_BIT lives in an already-present field's spare bit), keeping metadata-size at 24 bytes and avoiding any new allocations.\n"
}
```

### 17:28:55 chat

Three hypothesis files written to the output directory. Here's a summary of what was produced:

---

**H01 — Adaptive Protected Quota** (`cycle/0002/v001` parent)
Targets **hit-rate without per-op-time regression**. Adds two `u32` counters (`op_count`, `demotion_count`) to `LruCache` — zero node-layout change. Every ~`capacity` operations, `maybe_adapt()` checks whether demotion pressure is high (shrink `prob_cap` toward capacity/8, boosting hit-rate) or absent (relax back toward capacity/4, reducing protected-path cost). This directly tests the #1 open question from the cycle-0003 continuation: can dynamic quota sizing capture cycle/0003/v002's 0.729 hit-rate without its per-op-time penalty?

**H02 — CLOCK-Within-Protected via Referenced Bitmap** (`cycle/0002/v001` parent)
Targets **per-op-time** by eliminating the unlink+push_back_prot pointer writes on every protected hit. Adds a `Vec<u8>` bit-vector to `LruCache` (not to `Node`). Protected hits become a single bit-or; physical relinking only happens during the rare demotion event, where a bounded 8-iteration CLOCK sweep grants referenced nodes a second chance before demotion. Distinct from the failed MRU-tail guard: the sweep fires at demotion time (rare), not on every access, so there is no per-call branch overhead.

**H03 — Two-Hit Probationary Filter via SEEN_BIT** (`cycle/0002/v001` parent)
Targets **hit-rate** via a policy change with zero structural cost. Encodes a `SEEN_BIT` in spare bit 30 of the existing `prev` field (alongside `PROT_BIT` in bit 31), reducing `IDX_MASK` from 0x7FFF_FFFF to 0x3FFF_FFFF (still supports ~1 billion entries). A probationary first-hit refreshes the item's position in probationary and sets `SEEN_BIT`; only the second hit graduates it to protected. This filters scan artefacts that would otherwise briefly occupy protected slots.


---

- finished: 2026-07-19T17:28:55+00:00
- duration: 461.28s
- status: ok
- input_tokens: 12
- output_tokens: 25387
- cache_read_tokens: 468324
- cache_write_tokens: 64189
- cost_usd: 0.763593
