# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0009\hypotheses
- allow_edits: True
- started: 2026-07-20T06:06:33+00:00

## timeline

### 06:06:37 thinking

Let me read all the relevant context files to understand the current state of the research and what hypotheses have been tried before. I'll start by reading the key files in parallel.

### 06:06:38 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0008\\cycle-insight.md"
}
```

### 06:06:38 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0008\\cycle-winners.json"
}
```

### 06:06:39 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\leaderboard.md"
}
```

### 06:06:39 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cross-cycle-insights.md"
}
```

### 06:06:43 thinking

Good, now let me read the cycle 0008 variants to understand what was tried in the most recent cycle. Let me also read the baseline and the winning variants from prior cycles.

### 06:06:44 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0008\\variants"
}
```

### 06:06:45 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles-ledger.jsonl"
}
```

### 06:06:49 thinking

Now let me read the actual code for the key winning variants to understand the implementation details. Let me read the cycle 0008 v001 (current best), cycle 0007/v003 (hit-rate winner), and cycle 0008/v002 (protected-segment) overlay files.

### 06:06:50 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0008\\variants\\v001\\attempts\\01\\overlay\\src\\tiny_lfu\\mod.rs"
}
```

### 06:06:50 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0008\\variants\\v002\\attempts\\01\\overlay\\src\\tiny_lfu\\mod.rs"
}
```

### 06:06:54 thinking

Now let me read the winning variants from prior cycles to understand what was already tried - specifically cycle 0007/v003 (the hit-rate winner with 8x sketch width) and the insights from cycle 0008 v001, v002, and v003.

### 06:06:55 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0008\\variants\\v001\\insight.md"
}
```

### 06:06:56 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0008\\variants\\v002\\insight.md"
}
```

### 06:06:56 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0008\\variants\\v003\\insight.md"
}
```

### 06:06:59 thinking

Now let me also read the baseline code and the winning variants from cycles 0006 and 0007 to better understand the evolution.

Let me also read the cycle 0007/v003 overlay for the best-so-far high hit-rate variant.

### 06:07:00 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\tiny_lfu\\baseline\\src\\tiny_lfu\\mod.rs"
}
```

### 06:07:02 tool: Glob

```json
{
  "pattern": "**/v003/attempts/**/overlay/src/tiny_lfu/mod.rs",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0007"
}
```

### 06:07:04 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\tiny_lfu\\baseline\\src\\tiny_lfu\\mod.rs"
}
```

### 06:07:04 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0007\\variants\\v003\\attempts\\01\\overlay\\src\\tiny_lfu\\mod.rs"
}
```

### 06:10:26 thinking

Now I have a clear picture of the current state. Let me synthesize what I know to generate distinct hypotheses for Cycle 9.

**Current State Summary:**
- Best incumbent: `cycle/0007/v003` and `cycle/0008/v001` both at hit-rate=0.7409, metadata-size=48, per-op-time=100
- `cycle/0007/v003` = 8x sketch width factor (SKETCH_WIDTH_FACTOR: 8), u32 counters, with amortised strict admission gate (every 8th eviction), using remix-based hash approach. 

The v003 variant doubled p99 tail latency compared to baseline, though it still won overall. The v001 from cycle 0008 matched those metrics but used u16 counters instead, though put operations still showed p99 latency regression. Meanwhile, v002 experimented with a protected LRU segment at 20% capacity and baseline 4x sketch width, which added overhead through new struct fields and a protection flags vector, resulting in a 47% size increase to 224 bytes and slightly lower hit-rate at 0.7368.

For the next cycle, I'm thinking through several key optimizations: whether I can use a bitset instead of a Vec to store protection flags more compactly, what the ideal protected-segment ratio actually is since 20% was somewhat arbitrary, whether stacking the amortized admission gate on top of the protected-segment approach yields compounding benefits, exploring different window-to-main ratios since that's never been tuned, and whether two seeded hashes could cover all four rows with just one extra invocation through bit-slicing to reduce hash calls. The current best performer remains cycle 0007 v003 and cycle 0008 v001 at 0.7409 hit-rate. The cycle insights point me toward a compact protected-segment flag as the top priority—replacing the bloated `Vec<bool>` with a single `bool` field embedded directly in each `Node` struct, which eliminates the separate heap allocation and cuts the struct overhead by 72 bytes. I'm also considering combining this with an amortised gate strategy to further refine the eviction logic. ortised-gate baseline as the parent. I'd increase the protected capacity from 20% to 30% and move to a compact representation to reduce overhead.

The most promising approach seems to be embedding the protected flag directly in the Node structure instead of maintaining a separate Vec<bool>. This eliminates heap allocation, shrinks the cache struct, and keeps flag lookups cache-local with node access, which should address the main performance concern from the previous cycle.

I could also try combining the protected-segment logic with the amortised gate from the best incumbent, using the compact flag representation and keeping the 20% protected ratio—this would test whether those two mechanisms work well together. The key is keeping each experiment focused on one or two distinct changes so I can isolate what's actually driving improvements. I'm considering whether to test two independent hash seeds instead, which could cover multiple rows with fewer invocations—something predicted in earlier cycles but never properly evaluated. But that's a different experimental axis from the protected-segment tweaks. For H03, I could also explore the window-to-main ratio as a structural policy parameter, since eight cycles have touched various aspects of sketch structure but maybe not this particular lever in isolation.

Looking at what's actually feasible, I have a few concrete options: adjusting the protected ratio, implementing true independent hashing with dual seeds, using a compact bitset for flags, or trying lazy promotion where items need multiple hits before moving to protected. I'm leaning toward lazy promotion as the most distinct and valuable hypothesis to test next. The second option with adaptive protected ratio seems more promising since it introduces a genuinely new mechanism—adjusting the segment split based on how often items get demoted back to probationary. I'm leaning toward that for H03 rather than the dual-hash approach, which feels incremental compared to what's already been explored. Let me finalize my three hypotheses: H01 uses a compact flag to reduce memory overhead, H02 combines amortized gating with protected segments, and H03 will adapt the protected capacity dynamically based on demotion feedback. H03 adjusts the protected segment ratio to 30% instead of 20% to test whether a larger protected zone improves hit rates. But I'm realizing both H01 and H03 reference `cycle/0008/v002` as their parent, which isn't a cross-cycle winner—I need to ensure at least one hypothesis cites an actual cross-cycle winner like `cycle/0007/v003`, `cycle/0008/v001`, or `cycle/0006/v002`. ies this with `cycle/0007/v003`. H01 and H03 are distinct enough — one modifies how the flag is stored while the other adjusts the ratio — so they're testing different aspects of the protected-segment hypothesis from the same parent. I could also use `cycle/0008/v001` as a parent since it's a cross-cycle winner with the best overall score, though it's essentially `cycle/0007/v003` with different counter widths. The strongest approach is to make H01 use `cycle/0007/v003` as its parent and add the compact protected segment as the priority change.

For H02 and H03, I'll use `cycle/0008/v002` as the parent — H02 implements the compact flag representation directly to address the bloat issue, while H03 explores a different protected ratio at 30% of capacity. This setup gives me three distinct hypotheses: H01 compounds multiple mechanisms, H02 optimizes the same mechanism with a more efficient implementation, and H03 tunes the ratio parameter. Though H02 and H03 share the same parent, they're testing different dimensions of the protected-segment approach.

For H03 specifically, I should pivot toward something more orthogonal — maybe exploring the two-seeded-hash approach for true row independence rather than just tweaking the protected segment ratio. The cross-cycle insights point to this exact gap: whether two seeded hashes can cover all four rows efficiently using bit-slicing, which was predicted in Cycle 0003 but never cleanly evaluated against the bijective-remix baseline that took over afterward. Looking at the three approaches: the single-hash bit-slice from cycle 0003, the bijective-remix that became the current baseline in cycle 0005, and the two-seeded-hash bit-slice from cycle 0004 that uses independent hashes to cover pairs of rows. The key insight is that cycle 0006 showed remix chains can't replace independent seeds, but the specific combination of two genuine independent hashes with bit-slicing to reduce invocations was theorized in cycle 0003 but never properly tested against the bijective-remix winner — so the real question is whether that two-seeded approach can deliver both independence and efficiency.

The third hypothesis is to replace the bijective remix with a genuine second seeded hash for rows 2-3, giving true row independence while keeping the 8x width that avoids the p99 regression seen at 4x width, and testing whether the collision noise reduction at higher width makes this approach competitive.

Now I'm narrowing down the final hypothesis set: H01 targets moving the protected flag directly into the Node struct to eliminate heap allocation overhead, H02 combines the protected-segment logic with the amortised-gate and 8x sketch winner from the best cross-cycle performers, and H03 replaces the remix with a genuine seeded hash on one of the top incumbents.

I'm reconsidering the parent selection for H03 — using cycle/0008/v001 instead of cycle/0007/v003 introduces more diversity and lets me test whether true hash independence can offset the latency regression that u16 counters introduced, though I'm realizing this might be conflating two separate variables again.

Now I'm weighing two distinct directions for H03: either testing a smaller protected zone at 10% capacity to reduce demotional overhead, or going larger at 30% to see if that pushes hit rate higher despite more overhead. A third angle just occurred to me — tracking promoted nodes more compactly using a bit-field embedded in the HashMap value itself.

The challenge with that approach is finding spare bits in the existing node metadata. The current `Option<usize>` pointers for prev and next take up 8 bytes each, and since indices theoretically can't exceed 2^63, I could repurpose the high bit to store the protected flag without allocating extra memory. But this is getting complicated, so I'm going to stick with the clearer, more straightforward hypotheses instead.

For H01, I'm adding a simple `protected: bool` field directly to the Node struct and removing the separate `Vec<bool>` entirely. This cuts down on heap allocations and improves cache locality by keeping the flag colocated with the node data. For H02, I'm building on the 8x-width sketch winner from the previous cycle and layering in the protected-segment split with the compact flag, keeping the amortised gate logic intact.

Now I'm noticing H01 and H03 both start from v002 and add the compact flag, so they're duplicating that change. I should cleanly separate them: H01 handles just the compact flag optimization, while H02 compounds it with the other mechanisms.

But then H03 with the 30% ratio change while keeping the Vec<bool> storage doesn't really solve the struct bloat problem—that seems like a step backward. Let me reconsider the priorities here. The real open questions are whether the compact flag can actually solve the 47% growth issue, whether layering the amortised gate on top of the protected segment creates redundancy or compounds the benefits, and what the optimal protected-segment ratio actually is since 20% hasn't been properly tested against alternatives.

These are clearly independent concerns, so for H03 I should isolate just the ratio variable and test 30% against the baseline without mixing in flag storage changes. I'll go with 30% since it might capture more hot items, though it does reduce the probationary victim pool.

Refining my final selection: H01 moves the protected flag directly into each Node to eliminate the separate Vec<bool>, H02 layers the amortised admission gate on top of the protected-segment baseline to test for redundancy, and H03 tests a 30% protected ratio in isolation to find the optimal threshold. The three hypotheses are cleanly separated—H01 and H03 both modify cycle/0008/v002 in different ways, while H02 combines the best sketch and gate approach from cycle/0007/v003 with the protected segment logic. For H01, I'm moving the protected flag from a separate Vec into the Node struct itself, removing the prot_flags field entirely and updating all references to use the node's protected attribute instead. H02 starts from the cycle/0007/v003 baseline with its 8x sketch width and strict gate, then layers on the protected segment implementation with promotion and demotion logic. I need to verify the Node size calculations for H01 since adding the `protected: bool` field will affect memory layout, and I'm checking whether the fitness vector's 48-byte estimate accounts for this change properly. Option<usize> is actually 16 bytes on 64-bit systems since usize can be zero, so there's no niche optimization available. That gives us the full 48 bytes for Node: 8 (key) + 8 (value) + 16 (prev) + 16 (next). Adding a bool field would push it to 56 bytes with alignment, which trades the 24-byte Vec<bool> from the cache struct for an 8-byte increase per node—a net win if there are fewer than three nodes. Now I'm weighing this against the metadata-size fitness metric. So bumping Node to 56 bytes would worsen the metadata-size score. I need to find a way to track the protected flag without expanding Node itself — maybe by encoding it in a spare bit of an existing field, or better yet, using a separate bitset in the cache that packs multiple flags into each u64 to minimize overhead. So the bitset approach keeps the Node struct at 48 bytes, which preserves the metadata-size fitness score, while the Vec<u64> storage inside TinyLfuCache reduces overall memory usage. For H01, I should use this bitset approach with bit-level access operations rather than adding a bool field to Node.

For H02, I'm considering how to track protected nodes without modifying Node itself—the bitset approach is one option, but I need to explore other possibilities too. Adding a `protected` bool to Node would bump it from 48 to 56 bytes with padding, a 16.7% increase that would show up negatively in the metadata-size metric. Instead, I could use a bitset in the TinyLfuCache itself to track protected status without touching Node's layout, or get creative by encoding the protected flag in the high bit of the HashMap's stored indices.

The cleanest approach is using a `Vec<u64>` bitset for the protected-segment flags. So my final hypotheses are: replace the `Vec<bool>` with a bitset in cycle/0008/v002, add the same compact bitset representation to cycle/0007/v003 as a cross-cycle winner, and increase the protected-segment capacity to 30% in cycle/0008/v002.

Now I'm thinking through how to actually implement the protected segment in cycle/0007/v003 — I'd need to add the head, tail, count, and capacity fields to track the protected chain, plus the bitset for flags, and figure out the eviction logic that works with the existing amortized gate and single-chain LRU structure.

The real question is whether stacking a protected segment on top of the 8x width sketch actually compounds the hit-rate improvements, or if one benefit already captures most of what the other would provide. Implementing bitset operations... Adding protected segment initialization... Continuing the `get` method logic: after demoting a protected node if needed, I promote the accessed node to protected status and set its flag bit. For `put` with existing keys, I apply the same flag-checking logic. When the cache is full and needs eviction, if there are no probationary nodes left, I force-demote the LRU protected node while keeping the amortized eviction counter unchanged. New entries always start as probationary with their flag bit cleared, and I'm using bitwise operations on the flags array to test, set, and clear individual protection bits. Then I'm adjusting the protected segment capacity up to 30% in the cycle configuration. Let me verify that at least one hypothesis references a cross-cycle winner—H02 does by citing `cycle/0007/v003`, so the requirement is satisfied. Now I'm ready to write out the hypothesis files to the output directory.

### 06:10:31 chat

Now I have enough context. Let me synthesize and write the three hypotheses.

**State Summary:**
- Best incumbents: `cycle/0007/v003` and `cycle/0008/v001` at hit-rate=0.7409, metadata-size=48, per-op-time=100
- `cycle/0007/v003`: 8× sketch width, u32 counters, amortised strict gate (every 8th eviction uses `> victim+1`), bijective remix hash
- `cycle/0008/v001`: same as above but u16 counters (latency penalty didn't pay off)
- `cycle/0008/v002`: protected-segment (20% capacity), but `TinyLfuCache` bloated 47% (152→224 bytes) due to `Vec<bool> prot_flags`

**Three distinct strategies for Cycle 9:**
1. **H01** (compact bitset flags) — replace `Vec<bool>` with `Vec<u64>` bitset in `cycle/0008/v002`, reducing heap from 2048B → 32×8=256B at cap=2048, keeping Node at 48 bytes
2. **H02** (compound) — layer protected-segment (compact bitset) onto the best incumbent `cycle/0007/v003`, testing whether two independently validated mechanisms combine
3. **H03** (ratio probe) — increase protected capacity from 20%→30% in `cycle/0008/v002`, isolating the ratio variable

### 06:10:39 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0009\\hypotheses\\H01.md",
  "content": "# H01\n\nReplacing `prot_flags: Vec<bool>` with a packed `Vec<u64>` bitset in the protected-segment implementation will reduce the flag-array heap footprint 8× while preserving the segment-split's hit-rate gain and leaving `Node<u64,u64>` size at 48 bytes.\n\n## Parent\n\ncycle/0008/v002\n\n## Changes\n\nIn `TinyLfuCache`, change the field declaration from:\n\n```rust\nprot_flags: Vec<bool>,\n```\n\nto:\n\n```rust\nprot_flags: Vec<u64>,\n```\n\nIn `try_new`, change the initialiser from:\n\n```rust\nprot_flags: vec![false; capacity],\n```\n\nto:\n\n```rust\nprot_flags: vec![0u64; (capacity + 63) / 64],\n```\n\nReplace every read of `self.prot_flags[idx]` (boolean test) with the inline bit-test:\n\n```rust\n(self.prot_flags[idx / 64] >> (idx % 64)) & 1 != 0\n```\n\nReplace every write of `self.prot_flags[idx] = true` with the bit-set:\n\n```rust\nself.prot_flags[idx / 64] |= 1u64 << (idx % 64);\n```\n\nReplace every write of `self.prot_flags[idx] = false` (demotion path and victim reuse) with the bit-clear:\n\n```rust\nself.prot_flags[idx / 64] &= !(1u64 << (idx % 64));\n```\n\nAll four extra fields (`prot_head`, `prot_tail`, `prot_count`, `prot_capacity`) and all chain-management methods (`prot_unlink`, `prot_push_back`) remain byte-for-byte identical to `cycle/0008/v002`. The protected-zone ratio stays at `(capacity / 5).max(1)`. The admission gate comparison (`incoming_freq > victim_freq`) is unchanged.\n\nAt `capacity = 2048` the `Vec<bool>` occupied 2048 heap bytes; the `Vec<u64>` bitset occupies 32 × 8 = 256 heap bytes. `Node<u64,u64>` is not modified and remains 48 bytes.\n"
}
```

### 06:10:55 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0009\\hypotheses\\H02.md",
  "content": "# H02\n\nLayering a 20%-capacity protected LRU segment onto `cycle/0007/v003`'s amortised-gate and 8× sketch combination will push hit rate above the 0.7409 ceiling that either mechanism achieved alone.\n\n## Parent\n\ncycle/0007/v003\n\n## Changes\n\nStarting from `cycle/0007/v003` (8× sketch width, u32 counters, bijective remix hash, amortised strict gate), add the following protected-segment split using a compact `Vec<u64>` bitset for flag storage (so `Node<u64,u64>` stays at 48 bytes):\n\n**Struct additions** — add five fields to `TinyLfuCache`:\n\n```rust\nprot_head: Option<usize>,\nprot_tail: Option<usize>,\nprot_count: usize,\nprot_capacity: usize,\nprot_flags: Vec<u64>,   // packed bitset, one bit per node slot\n```\n\n**`try_new` additions** — initialise alongside existing fields:\n\n```rust\nprot_head: None,\nprot_tail: None,\nprot_count: 0,\nprot_capacity: (capacity / 5).max(1),\nprot_flags: vec![0u64; (capacity + 63) / 64],\n```\n\n**Add two chain-management helpers** operating on `prot_head`/`prot_tail` (structurally identical to the existing `unlink`/`push_back` methods but updating `prot_head`/`prot_tail` instead of `head`/`tail`):\n\n```rust\nfn prot_unlink(&mut self, idx: usize) { ... }\nfn prot_push_back(&mut self, idx: usize) { ... }\n```\n\nUse inline bit-test `(self.prot_flags[idx/64] >> (idx%64)) & 1 != 0` to check membership; `|= 1u64 << (idx%64)` to set; `&= !(1u64 << (idx%64))` to clear.\n\n**`get` method** — replace `self.unlink(idx); self.push_back(idx)` with:\n\n1. If the bit for `idx` is set (already protected): `prot_unlink(idx)` then `prot_push_back(idx)` (re-position at MRU end; `prot_count` unchanged).\n2. If clear (probationary): `self.unlink(idx); prot_count += 1`. If `prot_count > prot_capacity`, demote the LRU protected node (`prot_unlink(prot_head)`, clear its bit, `push_back(prot_head_idx)`). Then `prot_push_back(idx)` and set the bit.\n\n**`put` method — existing-key branch** — apply the same flag-based promotion logic as in `get` (unlink from whichever chain the existing node is in, promote/re-position in protected chain).\n\n**`put` method — full-cache eviction branch** — keep the existing `evict_counter`-based amortised gate logic unchanged. Add one guard before reading `self.head`: if `self.head.is_none()` (all slots in protected chain), force-demote `prot_head` (prot_unlink, clear bit, push_back, prot_count -= 1) so the probationary chain is non-empty before the gate comparison.\n\nNew node insertions (filling empty slots) always enter via `push_back` (probationary) with flag bit = 0.\n"
}
```

### 06:11:06 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0009\\hypotheses\\H03.md",
  "content": "# H03\n\nIncreasing the protected-segment capacity from 20% to 30% of total cache capacity in `cycle/0008/v002` will improve hit rate by shielding a larger fraction of the hot Zipfian working set from the eviction victim pool.\n\n## Parent\n\ncycle/0008/v002\n\n## Changes\n\nIn `try_new`, change the single line that initialises `prot_capacity` from:\n\n```rust\nprot_capacity: (capacity / 5).max(1),\n```\n\nto:\n\n```rust\nprot_capacity: (capacity * 3 / 10).max(1),\n```\n\nAt `capacity = 2048` this yields a protected zone of 614 slots (≈30%) instead of 409 (≈20%). The integer arithmetic `capacity * 3 / 10` uses truncating division, which is correct for this purpose.\n\nEvery other line of `cycle/0008/v002` — including `prot_flags: vec![false; capacity]`, the chain-management helpers (`prot_unlink`, `prot_push_back`), the promotion/demotion logic in `get` and `put`, the `Vec<bool>` flag storage, and the admission gate comparison (`incoming_freq > victim_freq`) — remains byte-for-byte identical to the parent.\n\nThis isolates the ratio variable: if hit rate improves, the protected zone was undersized at 20%; if hit rate regresses or the probationary victim pool becomes too shallow, 30% is too large and the optimal fraction lies below 30%.\n"
}
```

### 06:11:18 chat

Three hypotheses written to `cycles/0009/hypotheses/`:

---

**H01** — `cycle/0008/v002` parent  
**Claim:** Replace `prot_flags: Vec<bool>` with a packed `Vec<u64>` bitset to shrink the flag-array heap from 2048 B → 256 B (at cap=2048) while keeping `Node<u64,u64>` at 48 bytes and all protection semantics intact.  
**Changes:** Swap `Vec<bool>` → `Vec<u64>`, init with `vec![0u64; (capacity+63)/64]`, use inline bit-test/set/clear everywhere `prot_flags[idx]` is read or written. All chain management and the 20% ratio are unchanged.

---

**H02** — `cycle/0007/v003` parent (cross-cycle winner, required citation)  
**Claim:** Layering a 20%-capacity protected LRU segment onto the amortised-gate + 8× sketch winner will push hit rate above the 0.7409 ceiling that either mechanism achieved alone.  
**Changes:** Add `prot_head`, `prot_tail`, `prot_count`, `prot_capacity`, and `prot_flags: Vec<u64>` (bitset) to `TinyLfuCache`. Add `prot_unlink` / `prot_push_back` helpers. Rewrite `get` and the existing-key `put` branch to promote probationary→protected and re-position hot items inside the protected chain. Keep the amortised gate (`evict_counter & 7 == 0` strict comparison) unchanged. Guard the eviction path against an empty probationary chain by force-demoting `prot_head` when needed.

---

**H03** — `cycle/0008/v002` parent  
**Claim:** Increasing the protected-segment capacity from 20% to 30% (`capacity * 3 / 10`) will improve hit rate by shielding a larger fraction of the hot Zipfian working set from the victim pool.  
**Changes:** One line only — `prot_capacity: (capacity / 5).max(1)` → `(capacity * 3 / 10).max(1)`. Everything else is byte-for-byte identical to the parent.


---

- finished: 2026-07-20T06:11:18+00:00
- duration: 284.75s
- status: ok
- input_tokens: 11
- output_tokens: 16100
- cache_read_tokens: 370046
- cache_write_tokens: 45922
- cost_usd: 0.5263
