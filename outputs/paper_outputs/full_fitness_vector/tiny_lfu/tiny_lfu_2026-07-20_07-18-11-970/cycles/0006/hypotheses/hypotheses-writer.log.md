# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0006\hypotheses
- allow_edits: True
- started: 2026-07-20T05:14:17+00:00

## timeline

### 05:14:21 thinking

Let me read the context files to understand the current state of the research and then generate 3 distinct hypotheses for cycle 0006.

### 05:14:22 tool: Bash

```json
{
  "command": "ls \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0005\\variants\"",
  "description": "List cycle 0005 variants"
}
```

### 05:14:23 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0005\\cycle-insight.md"
}
```

### 05:14:23 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0005\\cycle-winners.json"
}
```

### 05:14:24 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\leaderboard.md"
}
```

### 05:14:24 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cross-cycle-insights.md"
}
```

### 05:14:33 thinking

Now let me read the cycle 0005 variants and the cycle 0004 winners to understand what we're working with.

### 05:14:35 tool: Glob

```json
{
  "pattern": "**/v*/eval/fitness-vector.json",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0005\\variants"
}
```

### 05:14:36 tool: Glob

```json
{
  "pattern": "**/v*/insight.md",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0005\\variants"
}
```

### 05:14:38 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0005\\variants\\v001\\insight.md"
}
```

### 05:14:39 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0005\\variants\\v002\\insight.md"
}
```

### 05:14:39 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0005\\variants\\v003\\insight.md"
}
```

### 05:14:40 tool: Glob

```json
{
  "pattern": "**/v*/attempts/*/overlay/**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0005\\variants"
}
```

### 05:14:42 thinking

Now let me read the actual overlay files to understand the code structure for the winning variants.

### 05:14:43 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0005\\variants\\v002\\attempts\\01\\overlay\\src\\tiny_lfu\\mod.rs"
}
```

### 05:14:43 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0004\\variants\\v001\\attempts\\01\\overlay\\src\\tiny_lfu\\mod.rs"
}
```

### 05:14:46 thinking

Now let me also read the baseline to understand the reference point, and check the cycles-ledger for a broader history.

### 05:14:47 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\tiny_lfu\\baseline\\src\\tiny_lfu\\mod.rs"
}
```

### 05:14:48 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles-ledger.jsonl"
}
```

### 05:14:53 thinking

Now let me read the cycle 0004 winners (v001 and v002) to get full context on what their implementations look like. I've already read v001 from cycle 0004. Let me also look at the cycle 0004 v002 (the amortized best-of-2 eviction).

### 05:14:53 chat

Now let me check the cycle 0004 winner v002 overlay:

### 05:14:54 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0004\\variants\\v002\\attempts\\01\\overlay\\src\\tiny_lfu\\mod.rs"
}
```

### 05:18:03 thinking

Now I have a thorough understanding of the codebase and the research history. Let me plan 3 distinct hypotheses for cycle 0006.

**Context Summary:**

The current best variants are:
- `cycle/0004/v001`: hit-rate=0.7372, metadata-size=48, per-op-time=100 (uses 2 seeded hashes → bit-sliced into 4 positions, no amortized eviction)
- `cycle/0005/v002`: hit-rate=0.737, metadata-size=48, per-op-time=100 (uses bijective remix h0→h1, no amortized eviction)
- `cycle/0004/v002`: hit-rate=0.7369, metadata-size=48, per-op-time=100 (uses single-hash bit-slice with amortized 1-in-8 best-of-2 eviction)

The open research threads point toward several promising directions: applying stricter admission thresholds only during amortized probe operations, tuning the window and segment sizing parameters, reusing pre-computed frequency estimates to reduce redundant probes, and exploring two-phase hashing approaches.

Looking at the actual code structure across the best variants, I'm noticing the lineages have diverged—cycle 0005 v002 uses bijective remix hashing without amortized eviction, while cycle 0004 v002 applies single-hash bit-slicing with the amortized best-of-2 strategy. The current incumbents represent different trade-offs between hash complexity and eviction sophistication.

What stands out is that merging the bijective remix approach with amortized eviction hasn't been directly tested yet, nor has the idea of deriving two independent remix steps from a single hash to eliminate the SipHash dependency entirely. There's also the possibility of reusing the frequency estimate from the insertion path rather than recalculating it during eviction. The first hypothesis is to combine bijective remix from cycle 0005 v002 with the amortized 1-in-8 best-of-2 eviction strategy from cycle 0004 v002—adding an eviction counter that triggers the comparison logic every eighth operation. These are orthogonal optimizations targeting different bottlenecks: bijective remix reduces hash serialization overhead while amortized eviction improves which candidate gets removed. The concern is that cycle 0005 v002 uses bijective remix whereas cycle 0004 v002 uses a single hash bit-slice, so I need to think through whether combining them actually makes sense or if there's a confound in the comparison.

The second hypothesis involves applying a softer admission gate only on the amortized-probe path, building on the existing frequency comparison in `put()`.  Instead of the stricter threshold that disrupted residency in c0005/v003 by firing on every eviction, I'd limit it to the 1-in-8 probe path with a saturating increment to the victim frequency—keeping the plain comparison for the other 7/8 of evictions. This targets admission control rather than just eviction candidate selection. I'm distinguishing between two separate approaches for the 1-in-8 probe path: one that combines amortized best-of-2 victim selection with the plain threshold, and another that applies a stricter admission gate only on those infrequent probes without changing victim selection. H03 takes a different angle entirely—using double bijective remix steps to remove SipHash from the estimation process altogether. values from just the raw key hash and two remix operations. The question is whether this eliminates the need for SipHash entirely while still providing four sufficiently independent row positions, though there's some uncertainty about whether this approach is actually safer than the current method that already uses SipHash plus remix. The remix approach gives better row independence than single-hash bit-slicing, so I could chain two remixes together—using one SipHash followed by two remix steps to extract windows for all four rows. That's only three extra ALU instructions compared to the current single-remix version, which might be worth it for improved inter-pair independence. But I need to check what c0004/v001 is actually doing with its two full hashes. remix approach could give us better position diversity than a single remix, potentially matching the hit rate of two SipHash calls while staying cheaper. I'm sketching out a few position extraction patterns—either pulling from different bit ranges across the original and remixed hashes, or reusing bit ranges from the same remix in different positions—to see which arrangement gives the most independent lookups.

The structural parameters angle is interesting too. Looking at the actual cache code, there's no window-protected-probationary segmentation at all—it's just a plain LRU with frequency-based admission. A proper TinyLFU would split the cache into those three regions with different promotion rules, but the current implementation skips that entirely. Implementing it would be a significant architectural shift, requiring a complete restructuring of how nodes are organized and promoted.

Given that hit rates have plateaued around 0.736-0.738 for several cycles, breaking through with a structural change like this could be worthwhile, though it's risky.

So my three hypotheses are: combining the bijective remix approach with amortized best-of-2 eviction, exploring different frequency decay schedules, and potentially implementing the window-protected-probationary segmentation. Let me flesh out the first one with specific implementation details. On the other 7/8 evictions, I'll stick with the standard admission check so the stricter gate only fires on the sampled path. The key insight is that decoupling residency effects from threshold tightening by limiting firing rate rather than softening the threshold itself makes this viable. For H03, I'm exploring double bijective remix by computing two seeded hashes and using the remixed value to improve row independence, building on the earlier finding that better hash independence correlates with hit-rate gains. Comparing the double remix approach against the current c0005/v002 baseline — the extra remix step adds another multiply-rotate-xor operation but might give better distribution across the four positions by using h0, h1, and h2 values instead of just splitting h0 and h1 in half. - pos1 = bits 0-15 of remix(h0)  [= h1]
- pos2 = bits 0-15 of remix(h1)  [= h2]
- pos3 = bits 0-15 of remix(h2)  [= h3]

This provides maximum spread between positions because each is derived from a full transformation of the previous. Compare to current where pos0 and pos1 come from the same hash value (h0) — they share all 48 high bits, which can cause false collisions.

This "Fibonacci chain" approach might give better independence. It's worth testing!

OK so my 3 hypotheses:

**H01**: Merge bijective-remix hash (c0005/v002) with amortized 1-in-8 best-of-2 victim selection (c0004/v002 mechanism)
→ hits both latency and hit-rate together
- Parent: `cycle/0005/v002`
- Target: hit-rate (expected slight improvement from better victim selection)

**H02**: Rate-limited strict admission gate — on 1-in-8 evictions only, require `incoming_freq > victim_freq.saturating_add(1)` to block marginal admissions
- Parent: `cycle/0005/v002`  
- Target: hit-rate (while limiting residency disruption to 12.5% of evictions)

**H03**: Four-stage bijective remix chain — use h0, remix(h0), remix²(h0), remix³(h0) as 4 distinct positions (one full bit width each) rather than two 16-bit windows from each of 2 values
- Parent: `cycle/0005/v002`
- Target: hit-rate (via better position independence)

All three hypotheses cite `cycle/0005/v002`, which is a cross-cycle winner, so the requirement is satisfied. Now I need to think through whether any should parent from a different incumbent — H01 is merging two lineages, so it should parent from the bijective remix base with changes from amortized eviction, which points back to `cycle/0005/v002`. Looking at the cycle 0005 variants, none of them actually combined the bijective remix with the 1-in-8 amortized eviction from cycle 0004/v002 — they either tightened the probe rate, swapped the hash function, or added a stricter gate separately. So H01 with both improvements together is genuinely untested. Now I'm reconsidering whether H

01 and H02 are sufficiently different: H01 improves victim selection by picking the lower-frequency entry between head and head.next, while H02 tightens the admission threshold to require `incoming > victim + 1` instead of just `incoming > victim`. They're targeting different failure modes — one makes eviction smarter, the other makes admission stricter. For H03, there's also the idea of reusing frequency estimates already computed during the put operation to potentially raise the amortization ceiling without adding latency. I'm looking at the actual implementation where we call `estimate` on both the victim and the incoming key right after incrementing. The issue is that if this is an update case where the key already exists, we return early before any eviction logic runs, so those estimate calls never happen. That's where the optimization opportunity lies—we're only computing these estimates when we actually need to evict.

The "reuse" idea seems to be about caching the frequency value during the increment step so we can reference it later in the eviction decision, rather than calling estimate multiple times. In the amortized eviction approach, most evictions skip the best-of-2 comparison entirely, but the sampled ones do two extra estimate calls for the head and its next node—that's where reusing a pre-computed frequency could help.

I'm settling on three main hypotheses to explore: bijective remix with amortized 1-in-8 best-of-2 victim selection, bijective remix with a stricter admission gate on the sampled path, and a four-step remix chain for better row independence. For the third one, I'll branch it from the two-hash variant instead of the same parent as the others, to get more diversity in the exploration.

The key insight for H03 is that I can achieve similar hash independence quality but at lower computational cost—using one SipHash plus two remix operations instead of two full SipHash calls. Let me trace through what that would look like starting from the two-hash approach. Comparing different bit-slicing strategies for the four positions. The 4-stage chain approach where each position draws from a successive bijective remix of the hash, all using the same bit range, gives better independence between positions than splitting a single hash value. I'm going with that strategy, which aligns with what I had in cycle 0005 version 2. gives me 1 SipHash plus 3 remix operations, which is only about 10 ALU instructions after the initial hash—much more efficient than calling SipHash twice, and it should provide better independence between the four positions than just bit-shifting within the same hash value.

I'm settling on three concrete hypotheses to test: the first adds a best-of-2 victim selection mechanism with an eviction counter that triggers every 8 operations, comparing frequencies of adjacent nodes to pick the lower one as the victim while keeping the bijective remix hash the same. The second hypothesis is still forming. These three hypotheses target different optimization axes—eviction selection, admission thresholds, and hash independence—so they should give me meaningful signal on what matters most. Now I need to write out the actual hypothesis files, though I'm realizing H03's parent might need adjustment since c0004/v001 already uses two seeded hashes and I'm proposing to replace one with a remix chain instead. = h0[0:15], pos1 = h1[0:15], pos2 = h2[0:15], pos3 = h3[0:15] where each subsequent hash is a remix of the previous one. This approach uses the same number of hash operations as the current version but chains the remixes together instead of extracting two bit windows from each hash, which eliminates the dependency on the higher bits. Parenting from c0005/v002 would make the description clearer since it directly replaces the dual bit-slice strategy with a four-stage remix chain.

Alternatively, I could parent from c0004/v001 and frame this as a computational optimization: instead of calling seeded_hash twice with different seeds, I'd compute just one hash and then apply three remix operations to generate the four distinct positions. This reduces the number of SipHash calls from two to one while still maintaining four independent position values.

Looking at the cross-cycle winners, c0004/v001 has a hit rate of 0.7372, which is slightly better than the other incumbents I'm considering. If I cite it as the parent for H03, I get the variety the instructions require while making a natural argument about trading one expensive hash operation for cheaper remix steps. 2 seeded hashes). This gives me two different parent lineages while keeping all three hypotheses diverse.

Now I'm drafting the hypothesis files. H01 will combine the bijective-remix hash from cycle/0005/v002 with the amortized victim selection strategy from cycle/0004/v002 to see if they work better together. I'm adding an evict counter field and modifying the eviction logic to sample victims every eighth operation...

Implementing victim selection logic...

The frequency comparison stays the same, and the bijective remix hash functions remain untouched from the parent. H02 is going to test a stricter admission threshold by raising the bar for incoming frequencies...

I'm adding an eviction counter field to track how often we're evicting, then using it to gate a stricter threshold check—only every eighth eviction will require the incoming frequency to exceed the victim's frequency plus one, while other evictions just need to beat the victim's frequency outright. This amortized approach should improve hit rates without causing too much disruption to the cache residency.

For the second variant, I'm replacing the second seeded hash call with a chain of bijective remix operations to maintain the hit-rate benefits while avoiding the computational cost of a second SipHash invocation. Defining claim statements...

I'm verifying the output directory path exists and is set as my current working directory, then I'll write out the hypothesis files.

Now I'm checking the counter logic in the H01 implementation to make sure it's clean—specifically looking at how the eviction counter increments and determines which victim to evict based on frequency estimates.

For H02, the counter needs to increment before the threshold calculation, and the victim selection always defaults to the LRU head. Let me trace through...

Writing eviction logic...

For H03, I'm replacing the 2-hash bit-slice approach with a 4-stage remix chain, which should make the Changes section clearer. I need to verify whether the earlier cycle has the seeded_hash implementation.

### 05:18:03 chat

Now I have enough context. Let me synthesize the 3 hypotheses and write them.

### 05:18:12 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0006\\hypotheses\\H01.md",
  "content": "# H01\n\nCombining the bijective-remix hash from cycle/0005/v002 with the amortized 1-in-8 best-of-2 victim selection from cycle/0004/v002 will improve hit rate over either mechanism applied in isolation.\n\n## Parent\n\ncycle/0005/v002\n\n## Changes\n\nAdd a `evict_counter: u32` field to `TinyLfuCache`. Initialise it to `0` in `try_new`. In `put()`, in the full-cache eviction branch, replace the single plain `self.head.expect(...)` victim selection with the following:\n\n```rust\nself.evict_counter = self.evict_counter.wrapping_add(1);\nlet victim_idx = if self.evict_counter & 7 == 0 {\n    // Every 8th eviction: pick the lower-frequency of LRU head vs second-oldest.\n    let head_idx = self.head.expect(\"TinyLfuCache: full cache has no victim\");\n    match self.nodes[head_idx].next {\n        None => head_idx,\n        Some(second_idx) => {\n            let head_freq = self.sketch.estimate(&self.nodes[head_idx].key);\n            let second_freq = self.sketch.estimate(&self.nodes[second_idx].key);\n            if second_freq < head_freq { second_idx } else { head_idx }\n        }\n    }\n} else {\n    // 7 out of 8 evictions: plain LRU-head selection, no extra sketch probes.\n    self.head.expect(\"TinyLfuCache: full cache has no victim\")\n};\n```\n\nEverything that follows (clone victim key, compute `victim_freq`, compute `incoming_freq`, compare `incoming_freq > victim_freq`, swap the node) is identical to the parent. The `seeded_hash`, `remix`, `increment`, and `estimate` functions are left entirely unchanged from cycle/0005/v002.\n\n## Rationale\n\ncycle/0004/v002 demonstrated that 1-in-8 best-of-2 victim selection improves hit rate at latency-neutral cost when built on the single-hash bit-slice base. cycle/0005/v002 independently replaced the second SipHash with a cheaper bijective remix while preserving hit rate and per-op latency. These two mechanisms target independent failure modes — one corrects frequency-blind victim selection, the other reduces serial hash dependency — and have never been combined. The hypothesis is that stacking them on the bijective-remix base yields additive hit-rate benefit without a latency regression, since the amortization rate (1-in-8) is the proven latency-neutral ceiling.\n"
}
```

### 05:18:26 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0006\\hypotheses\\H02.md",
  "content": "# H02\n\nApplying the stricter admission threshold (`incoming_freq > victim_freq.saturating_add(1)`) only on the 1-in-8 amortized eviction path will raise hit rate relative to the plain-threshold parent without the full residency disruption seen when the stricter gate fires on every eviction.\n\n## Parent\n\ncycle/0005/v002\n\n## Changes\n\nAdd a `evict_counter: u32` field to `TinyLfuCache`. Initialise it to `0` in `try_new`. In `put()`, in the full-cache eviction branch, keep victim selection as the plain LRU head (unchanged from parent) but modulate the admission threshold by the counter:\n\n```rust\nself.evict_counter = self.evict_counter.wrapping_add(1);\nlet victim_idx = self.head.expect(\"TinyLfuCache: full cache has no victim\");\nlet victim_key = self.nodes[victim_idx].key.clone();\nlet victim_freq = self.sketch.estimate(&victim_key);\nlet incoming_freq = self.sketch.estimate(&key);\nlet admit = if self.evict_counter & 7 == 0 {\n    // Every 8th eviction: require the incoming key to beat the victim by at least 1.\n    incoming_freq > victim_freq.saturating_add(1)\n} else {\n    // Remaining 7 out of 8: standard \"> victim\" comparison.\n    incoming_freq > victim_freq\n};\nif admit {\n    self.map.remove(&victim_key);\n    self.unlink(victim_idx);\n    self.nodes[victim_idx].key = key.clone();\n    self.nodes[victim_idx].value = value;\n    self.push_back(victim_idx);\n    self.map.insert(key, victim_idx);\n}\n```\n\nNo change to victim selection (still always the LRU head), no best-of-2 probe, and the bijective remix hash functions (`seeded_hash`, `remix`, `increment`, `estimate`) are unchanged.\n\n## Rationale\n\ncycle/0005/v003 showed that `> victim_freq.saturating_add(1)` on every eviction raises hit rate but doubles p50/p99 latency by altering which nodes occupy hot cache lines — a residency effect independent of execution overhead. The cycle insight identified the fix as limiting the gate's firing rate rather than softening the threshold value. Firing the strict gate on only 1-in-8 evictions limits residency churn to one-eighth of its prior frequency while still blocking the most clearly marginal admissions, preserving a portion of the hit-rate signal. No best-of-2 victim comparison is added, so this hypothesis is structurally distinct from H01 even though both add an `evict_counter`.\n"
}
```

### 05:18:41 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0006\\hypotheses\\H03.md",
  "content": "# H03\n\nReplacing the second seeded-hash call in cycle/0004/v001 with a three-step bijective remix chain will preserve its hit-rate advantage over single-hash variants while eliminating one SipHash invocation from every sketch operation.\n\n## Parent\n\ncycle/0004/v001\n\n## Changes\n\nAdd the `remix` function (from cycle/0005/v002) immediately after `seeded_hash`:\n\n```rust\n#[inline(always)]\nfn remix(h: u64) -> u64 {\n    h.wrapping_mul(0x9e3779b97f4a7c15_u64)\n        .rotate_right(32)\n        ^ h\n}\n```\n\nIn `CountMinSketch::increment` and `CountMinSketch::estimate`, replace the two-call computation (`h0 = seeded_hash(key, 0)`, `h1 = seeded_hash(key, 1)`) and the existing four 16-bit-window positions with a four-stage remix chain where each position takes the low bits of a successive full-width bijective transformation:\n\n```rust\n// Replace this (parent):\nlet h0 = seeded_hash(key, 0);\nlet h1 = seeded_hash(key, 1);\nlet mask = self.width - 1;\nlet pos0 = (h0 as usize) & mask;\nlet pos1 = ((h0 >> 16) as usize) & mask;\nlet pos2 = (h1 as usize) & mask;\nlet pos3 = ((h1 >> 16) as usize) & mask;\n\n// With this:\nlet h0 = seeded_hash(key, 0);\nlet h1 = remix(h0);\nlet h2 = remix(h1);\nlet h3 = remix(h2);\nlet mask = self.width - 1;\nlet pos0 = (h0 as usize) & mask;\nlet pos1 = (h1 as usize) & mask;\nlet pos2 = (h2 as usize) & mask;\nlet pos3 = (h3 as usize) & mask;\n```\n\nRemove the `seeded_hash(key, 1)` call. All cache policy logic, the `TinyLfuCache` struct, and `seeded_hash` remain unchanged from cycle/0004/v001.\n\n## Rationale\n\ncycle/0004/v001's two-seeded-hash approach achieved the highest hit rate of any incumbent (0.7372) by providing genuine pairwise independence between row pairs. cycle/0005/v002 showed that a single Fibonacci multiply-rotate-XOR remix is sufficient for row-pair independence at the cost of intra-pair correlation (pos0 and pos1 share all but 16 bits of h0). The three-step chain gives each of the four row positions a distinct full-width hash value — each derived from a complete bijective transformation of the previous — providing better inter-row independence than the 2×16-bit-window approach while costing three ALU instructions rather than a second SipHash invocation. The goal is to match or exceed cycle/0004/v001's hit rate at lower hash-computation cost.\n"
}
```

### 05:18:55 chat

Three hypothesis files written to the output directory. Here's what each one proposes and why they form a diverse cohort:

---

**H01** (`cycle/0005/v002` parent) — **Eviction candidate selection**  
Adds the proven 1-in-8 best-of-2 victim-frequency comparison from `cycle/0004/v002` onto the bijective-remix hash base from `cycle/0005/v002`. These two mechanisms were developed in separate lineages and have never been combined. The `evict_counter & 7 == 0` branch selects `min(head, head.next)` by frequency; the other 7/8 evictions use plain LRU-head. Targets hit-rate improvement while preserving per-op-time=100.

**H02** (`cycle/0005/v002` parent) — **Rate-limited admission threshold**  
Adds the same `evict_counter` but uses it to gate the *admission threshold*, not victim selection: on every 8th eviction, require `incoming_freq > victim_freq + 1` (the strict gate from c0005/v003); the other 7/8 use the standard `incoming_freq > victim_freq`. The cross-cycle insight identified that the strict gate's p50 doubling came from residency churn proportional to its firing rate — firing it at 12.5% instead of 100% should limit that churn while capturing some hit-rate benefit.

**H03** (`cycle/0004/v001` parent) — **Four-stage bijective remix chain**  
Replaces `seeded_hash(key, 1)` with three remix steps: `h1=remix(h0)`, `h2=remix(h1)`, `h3=remix(h2)`, using the low bits of each full-width value as the four row positions. This eliminates one SipHash call while giving every row a fully independent bijective transform rather than two 16-bit windows from the same hash — targeting `cycle/0004/v001`'s best-in-class hit rate (0.7372) at lower compute cost.


---

- finished: 2026-07-20T05:18:55+00:00
- duration: 278.03s
- status: ok
- input_tokens: 11
- output_tokens: 16562
- cache_read_tokens: 309172
- cache_write_tokens: 39704
- cost_usd: 0.491655
