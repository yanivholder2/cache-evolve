# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0004\hypotheses
- allow_edits: True
- started: 2026-07-20T04:53:29+00:00

## timeline

### 04:53:32 thinking

Let me read all the context files to understand the current state of the research and formulate hypotheses for cycle 0004.

### 04:53:33 tool: Bash

```json
{
  "command": "find \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0003\\variants\" -type d -maxdepth 1",
  "description": "List variant directories in cycle 0003"
}
```

### 04:53:34 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0003\\cycle-winners.json"
}
```

### 04:53:34 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0003\\cycle-insight.md"
}
```

### 04:53:36 thinking

Now let me read the variant overlays and insights from cycle 0003.

### 04:53:38 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0003\\variants\\**\\insight.md"
}
```

### 04:53:38 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\leaderboard.md"
}
```

### 04:53:39 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cross-cycle-insights.md"
}
```

### 04:53:41 thinking

Now let me read the cycle 0003 variant overlays to understand the actual code changes made.

### 04:53:43 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0003\\variants\\v001\\insight.md"
}
```

### 04:53:43 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0003\\variants\\v002\\insight.md"
}
```

### 04:53:44 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0003\\variants\\v003\\insight.md"
}
```

### 04:53:47 thinking

Now let me read the actual overlay code for the cycle 0003 variants to understand the code structure.

### 04:53:48 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0003\\variants\\v002\\attempts\\**\\*"
}
```

### 04:53:49 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0003\\variants\\v003\\attempts\\**\\*"
}
```

### 04:53:51 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0003\\variants\\v002\\attempts\\01\\overlay\\src\\tiny_lfu\\mod.rs"
}
```

### 04:53:51 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0003\\variants\\v003\\attempts\\01\\overlay\\src\\tiny_lfu\\mod.rs"
}
```

### 04:53:54 thinking

Now let me also read the baseline code to understand the full picture.

### 04:53:55 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\tiny_lfu\\baseline\\**\\*"
}
```

### 04:53:56 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles-ledger.jsonl"
}
```

### 04:57:11 thinking

Now I have a thorough understanding of the research history. Let me synthesize the key insights and formulate 3 hypotheses for cycle 0004:

**Current state:**
- **Incumbent winner**: `cycle/0003/v002` (hit-rate=0.7367, metadata-size=48, per-op-time=100)
  - Uses single-hash bit-slicing: one `hash_key()` call, positions derived via `(h >> (row * 16)) & (width - 1)` for rows 0-3
  - This eliminated 3 redundant hash invocations, cutting median get latency
- **Weakness**: 4 non-overlapping 16-bit windows from one 64-bit value provide weaker per-row independence → slight tail regression

**v003 from cycle 0003**: hit-rate=0.7384 (best ever!), metadata-size=48, per-op-time=200
  - Best-of-2 victim selection: compares LRU head vs second entry, evicts lower-frequency
  - Problem: fires two extra sketch probes on every full-cache `put` → latency too high
  - The hit-rate benefit is real but latency cost makes it a loser

**Cross-cycle insights:**
1. Two-phase hashing: two seeded hashes, each sliced into two row positions — may recover row independence while halving hash cost
2. Amortized/deferred best-of-2 eviction: sample only a fraction of evictions
3. Policy-level tuning: admission/eviction parameters (window size, protected/probationary ratio) — unexplored

**Closed axes:**
- Sketch decay at any cadence
- Node layout changes
- Sketch structure simplification (flat allocation, depth reduction)

**Open axes:**
1. Two-phase hashing (explicitly called out as the next step from v002)
2. Cheaper best-of-2 eviction (explicitly called out from v003)
3. Policy-level parameters (admission/eviction window ratios)

Now I'm formulating three distinct hypotheses to test next, starting with the two-phase hashing approach from the previous cycle.

**Hypothesis 1**: Two-phase hashing replaces the single hash with two seeded variants, each producing two non-overlapping windows, giving four positions with better independence while cutting hash cost in half compared to four independent hashes. This targets both hit-rate recovery and maintaining the latency improvements from the prior version.

**Hypothesis 2**: Sampled best-of-2 eviction applies a cheaper variant of the eviction strategy by sampling only a subset of candidates rather than evaluating all options.

**Hypothesis 3**: I'm reconsidering whether sketch counter reduction is even relevant—the metadata-size fitness dimension measures Node bytes, not the sketch heap itself, so that axis might not be the right lever to pull.

Looking at the leaderboard data, the 24-byte node winner from v002 achieved compaction through NonZeroU32 encoding, but the cross-cycle notes suggest this doesn't actually improve the overall score despite the size reduction.

I'm pivoting to explore a different angle: what if I adjust the admission threshold logic? Right now it's a strict inequality where incoming frequency must exceed victim frequency, but I could introduce some headroom or a frequency-based door mechanism to be more selective about what gets admitted. I'm considering whether a window-based approach like W-TinyLFU would work here — splitting the cache into a small LRU window area that's protected from frequency-based eviction, plus a main area with stricter admission rules. Since the core logic lives in the overlay, this structural change is feasible.

But I'm also exploring simpler alternatives: maybe just skipping the admission check for very hot incoming items, or looking at sketch counter memory efficiency — the current four-row sketch structure takes up significant space relative to the cache capacity, which could impact locality. That checks out. Using u8 instead of u32 for the counters would shrink it down to 16KB, making it much more cache-friendly, though I'd be trading off precision since u8 maxes out at 255. For typical LFU access patterns that might be fine, but I need to be careful—the notes mention that structural simplifications have backfired before, though reducing counter precision is different from changing the sketch geometry itself. The real concern is whether items accessed millions of times would saturate the u8 counters too quickly. be inflated by collisions with hot items, which could hurt admission accuracy—though that's just speculation without empirical validation. I'm also considering a different angle: what if I adjust the sketch width factor itself? Currently it's set to 4, so width equals capacity times 4. I could experiment with larger multipliers like 8 or 16 to reduce collisions at the memory cost, or smaller ones like 2 to ease cache pressure while tolerating more collisions. The tradeoff is that wider sketches consume more memory and potentially hurt cache locality, but the baseline already uses a 64KB sketch for capacity 1024 which seems important for collision suppression. Looking at the actual results, the baseline achieves 0.736 hit-rate while v002 gets 0.737—a marginal improvement.

But I think I'm overcomplicating this. The cycle insight explicitly identifies three open questions worth pursuing: two-phase hashing with multiple seeded hashes and row slicing, amortized or deferred best-of-2 eviction through batching, and admission/eviction policy parameters like window size and protected/probationary ratios. Let me focus on those instead. For the policy parameters, I'll try a W-TinyLFU-style approach with a small window area

—maybe 1% of capacity—that's exempt from frequency-based eviction and acts as a cold-start buffer before items compete in the main cache. Though actually, W-TinyLFU is a pretty large structural change. Let me think about how to specify this concretely: the current implementation has a flat node pool with an LRU linked list and evicts by comparing the head node's frequency against incoming frequency, but W-TinyLFU would need a separate window linked list for pure LRU ordering and a main area split between protected and probationary regions.

That's getting complex, so let me consider simpler alternatives—maybe a clock-style second-chance mechanism where the head item gets another chance if its frequency exceeds a threshold, or even just admitting everything without frequency checks during a warmup period until the sketch has seen enough accesses. I should also look more carefully at parameter tuning, like what the `SKETCH_WIDTH_FACTOR = 4` is actually doing.

Doubling it to 8 would reduce collision rates and improve frequency estimation, though it'd increase sketch cache pressure. Alternatively, I could try increasing `SKETCH_DEPTH` from 4 to 6 to suppress collisions further—though the insight about "four-row depth is load-bearing" suggests it's already well-tuned. But that only says depth reduction fails, not that increasing it would fail too. The three most promising hypotheses are the ones explicitly mentioned, starting with two-phase hashing. — larger sketch means more memory at construction and more cache misses per probe, but 128KB would still fit comfortably in L2 cache for a capacity of 1024 items. The real question is whether collisions are actually limiting hit-rate at the current width of 4096 buckets, given that we'd only have a 25% load factor with Zipfian access patterns.

Actually, the sampled eviction approach from v003 is more promising — that version hit 0.7384 hit-rate, the best we've seen, even though per-op-time doubled to 200. If we could apply best-of-2 eviction only on a fraction of evictions, say every 4th one, we'd get most of the hit-rate benefit while keeping the performance cost much lower.

For the third angle, I'm exploring probabilistic admission — instead of the hard threshold `incoming_freq > victim_freq`, what if we admit with a probability based on the frequency ratio? This would smooth out the admission decisions and potentially improve hit-rate without the sharp latency spikes.

But I'm realizing there's a deeper issue with the LRU ordering itself. The current promotion strategy moves accessed items to the MRU tail, which should protect hot items from eviction. Yet newly arriving cold items can still push hot items toward the LRU head through repeated accesses. Maybe the real problem isn't admission or probing, but how we're ordering items for eviction in the first place.

Let me focus on three concrete hypotheses to test. The first involves improving the hash independence in the sketch by using two seeded hashes instead of bit-slicing a single hash, which should give better row independence and more reliable frequency estimates.

The second hypothesis is exploring a probabilistic best-of-2 eviction strategy where I occasionally check a second LRU candidate during eviction, not just the oldest one.  I'd add a counter to the cache that samples every Kth eviction (say, every 8th), applying the more expensive best-of-2 logic only then while keeping the fast LRU-head path for the rest. This trades a small per-op cost for most of the hit-rate gains that the full best-of-2 approach gives.

For the third hypothesis, I'm reconsidering whether to double the sketch width to reduce hash collision noise, but I need to make sure at least one of my three variants cites a cross-cycle winner rather than all three building on the same parent.

Actually, I could use a simpler approach: reduce the sketch counter saturation by using u8 counters that max out at 255 instead of u32::MAX, which would keep the sketch more responsive to recent frequency changes while using less memory.

But there's a tradeoff here — in a Zipfian distribution, the most popular items get accessed thousands or millions of times and would saturate quickly at 255, losing the ability to distinguish between them. CMS relies on saturation to prevent overflow, but u32::MAX essentially gives unlimited headroom, whereas u8 forces early saturation that could hurt accuracy for relative frequency estimation.

Let me explore a different angle for H03 instead. I've already tested reducing sketch depth (didn't work, 4 is necessary), varying width, decay intervals, and hash computation. The main unexplored area is whether I can increase sketch width per row with power-of-two scaling.

Actually, I should focus on admission threshold tuning instead. I'll modify the admission logic to add a margin—requiring incoming items to be at least 2 more frequent than the victim they'd displace, rather than just 1 more. This should reduce churn from borderline cases where the frequency difference is just noise in the sketch, helping preserve warm items already in the cache.

The baseline and current version both use strict greater-than comparison, so tightening it to require an extra margin makes sense for stability. I need to be careful about overflow though—since `victim_freq` is a u32 and adding 1 could theoretically overflow at MAX_COUNT, though saturation should handle it safely.

The key insight is that with a margin of 1, I'd reject admissions where the incoming item is only marginally hotter than the victim, which helps prevent churn on stable working sets but risks blocking genuinely hot new items that are just slightly above the threshold. This seems worth testing. Let me nail down my three concrete hypotheses to explore.

For the first one, I'm replacing the single hash function with a two-phase approach: compute two independent hashes using different seeds, then extract four 16-bit slices from them to populate the four rows of the sketch. This gives each pair of rows a different seed, recovering independence while cutting hash invocations in half compared to the earlier baseline. I can remove the standalone hash function since it's no longer needed.

Now looking at the next optimization, I'm adding a sampled best-of-2 victim selection strategy to the TinyLfuCache. I'll track evictions with a counter and every 8th eviction, instead of just evicting the LRU head, I'll compare it against the second-oldest entry and evict whichever has lower frequency. The remaining 7 out of 8 evictions stick with simple LRU, keeping the per-operation overhead minimal while still improving hit rates.

For the incumbent bias approach, I'm tightening the admission gate by requiring incoming items to exceed the victim's frequency by at least 2, not just 1. This prevents thrashing when frequencies are close together—in a stable Zipfian distribution, those borderline cases are usually noise rather than genuine hot items. I need to watch for overflow when victim_freq approaches the maximum, though saturating arithmetic should handle that safely. These three hypotheses are sufficiently distinct: H01 targets hash computation efficiency to improve per-operation time, H02 uses sampled eviction to reduce hit-rate loss with controlled latency overhead, and H03 tightens admission thresholds to improve hit-rate through policy changes. Each one addresses a different mechanism and aligns with the research roadmap from prior cycles.

Now I'm writing out the three hypothesis files with their specific implementation details — which hashing scheme and constants for H01, what field to add and when to apply sampling for H02, and what admission check to modify for H03. I should also verify the parent citations: H01 and H02 both reference the cross-cycle winner from cycle/0003/v002, which satisfies the requirement that at least one hypothesis cites a proven incumbent, while H03 can parent from baseline.

Actually, I'm reconsidering whether H03 should parent from v002 instead of baseline. The admission check in v002 is the same as the original baseline, so either works technically. But keeping H03 as a baseline variant gives me better diversity in the cohort — it avoids having both H02 and H03 derived from v002, which would make the group too v002-heavy. A clean baseline comparison point is more valuable here.

I'm checking the victim selection logic in v002's put function to make sure I understand the current implementation before finalizing the hypotheses. I'm finalizing the three hypothesis claims now. H01 focuses on using dual seeded hashes in CountMinSketch to restore per-row independence and boost hit rate while keeping the performance gains from the current version. H02 is about sampling best-of-2 victim selection every eighth eviction instead of constantly, which should preserve most of the hit-rate improvement without sacrificing per-op-time. H03 targets reducing churn on stable working sets by requiring incoming items to exceed victim frequency by at least two counts before eviction.

Now I'm reconsidering H03's logic — making admission stricter means fewer new items get in, which could hurt hit rate for hot newcomers, but it also means incumbent items stay longer and get more hits. For a stable LIRS trace where the cache is already filled with hot items, the admission check mainly filters out cold items trying to enter, and those would be rejected anyway.

The real effect happens in the borderline zone where incoming frequency is close to the victim's frequency. With estimation noise from the sketch, a margin of one count helps reject these noisy borderline cases and keep incumbents, which are more likely to be hot than newcomers on a stable working set.

On traces with shifting hot items over time, this conservative approach might hurt, but for the LIRS trace it should help. Now I'm ready to write out the hypotheses and generate the output files.

For H02, I need to track evictions specifically. I'm going to add an `evict_counter` field to TinyLfuCache rather than deriving it from existing values—it's cleaner that way. The counter should increment when we actually evict something during a put operation, not on every put call.

For H01 and the two-phase hashing approach, I need to figure out how to derive two separate hash values from the existing hash function...

I'll create a seeded version that takes an additional parameter, then use seeds 0 and 1 to generate h0 and h1. The increment function will then use h0 for the first row and h1 for subsequent rows to get independent hash positions across the depth dimension. So the key trade-off here is that this approach uses 2 hash calls instead of v003's 4, while still maintaining 2 independent seeds unlike v002's single seed. It's essentially a middle ground between the two versions. Let me clarify the specific changes I'm making relative to v002: I'm keeping the `hash_key()` function but adding a seeded variant, and replacing the single hash with slicing in `increment()` with two separate seeded hash calls. Writing hash extraction code...

The cleanest approach is to create a helper function that computes both hashes and extracts all four positions at once, then reuse it in both the increment and estimate methods. This avoids duplicating the two-phase hashing logic and keeps the main methods clean.

I think I've worked through the design enough—time to write up the hypothesis files with the algorithm details spelled out conceptually.

### 04:57:11 chat

Now let me read the baseline source code to understand the starting point for H03:

### 04:57:12 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\tiny_lfu\\baseline\\src\\**\\*"
}
```

### 04:57:14 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\tiny_lfu\\baseline\\**\\*.rs"
}
```

### 04:57:16 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\tiny_lfu\\**\\*"
}
```

### 04:57:18 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\tiny_lfu\\src\\tiny_lfu\\mod.rs"
}
```

### 04:57:33 thinking

Now I have all the context I need. Let me write the three hypothesis files.

Summary of what I know:
- **cycle/0003/v002** (incumbent): uses single-hash bit-slicing for sketch positions — `h = hash_key(key)`, then `pos = (h >> (row * 16)) & (width-1)` for rows 0-3. This is faster than 4 independent seeded hashes but may have weaker row independence.
- **cycle/0003/v003**: used best-of-2 victim selection — compare head vs second oldest, evict lower-frequency. 

The v003 variant achieved a hit rate of 0.7384 (the best so far) but at the cost of doubled latency. The baseline approach uses seeded hashing with the row number as the seed, which is simpler but less efficient. Looking at the cross-cycle and cycle-specific insights, the winning strategy combines two-phase hashing from v002 (two seeded hashes covering 2 rows each), cheaper best-of-2 eviction using sampling or deferred updates, and policy parameters from the baseline. The key distinction with H01 is that v002 applies an unseeded hash function first.

H01 proposes using 2 seeded hash calls instead of the baseline's 4, where seed 0 covers rows 0-1 and seed 1 covers rows 2-3, with each seed extracting 2 row positions from 16-bit windows. This halves the hash computation overhead compared to baseline while maintaining independent coverage across row pairs. Now I need to write out the hypothesis files and verify the sketch depth parameter.

### 04:57:33 chat

Now I have everything I need. Let me write the three hypothesis files.

### 04:57:50 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0004\\hypotheses\\H01.md",
  "content": "# H01\n\nUsing two seeded hashes — each sliced into two row positions — instead of one unseed hash sliced into four windows will recover per-row independence and improve hit rate while preserving the per-op-time gain of cycle/0003/v002.\n\n## Parent\n\ncycle/0003/v002\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, replace the single-hash bit-slice scheme in `CountMinSketch` with a two-phase hashing scheme that uses exactly two seeded hash calls.\n\n**Remove** the `hash_key` free function (or make it private and unused), since the new scheme no longer needs an unseeded hash.\n\n**Add** a private helper (or inline the logic) that computes two seeded 64-bit digests. Each seed should be injected the same way cycle/0003/v003's `position()` did it — by calling `hasher.write_u64(seed)` before `key.hash(&mut hasher)` — so that `DefaultHasher` sees a different initial state for each seed:\n\n```\nh0 = seeded_hash(key, seed=0)   // covers rows 0 and 1\nh1 = seeded_hash(key, seed=1)   // covers rows 2 and 3\n```\n\n**Rewrite `CountMinSketch::increment` and `estimate`** to use these two digests. Derive the four row positions without a generic loop, since the SKETCH_DEPTH=4 constant makes the layout fixed:\n\n- row 0 position: `(h0 as usize) & (self.width - 1)`\n- row 1 position: `((h0 >> 16) as usize) & (self.width - 1)`\n- row 2 position: `(h1 as usize) & (self.width - 1)`\n- row 3 position: `((h1 >> 16) as usize) & (self.width - 1)`\n\nWithin each hash the two windows are derived by non-overlapping 16-bit slices of a 64-bit value (same as v002), but the two hashes carry independent seeds so rows 0–1 and rows 2–3 are statistically independent. This recovers the pairwise independence that four separate hashes provided, while still cutting hash invocations from four to two.\n\nPreserve all other logic: the `TinyLfuCache` struct and its `put`/`get` paths, the admission check (`if incoming_freq > victim_freq`), the LRU link operations, and all tests. Only `CountMinSketch::increment` and `CountMinSketch::estimate` (and any helper they call) should change.\n\n## Rationale\n\nv002 eliminated three redundant hash calls by deriving all four row positions from a single 64-bit value, yielding a measurable median-latency win. The cycle-0003 insight observed that the four 16-bit windows of one hash value provide weaker per-row independence than four independently seeded hashes, which showed up as slight tail regressions on write-heavy and collision-sensitive paths. Two seeded hashes is the natural midpoint: it halves the hash work compared to the original baseline while giving rows 0–1 and rows 2–3 genuinely independent seeds, reducing sketch estimation noise without re-introducing the full serial hash overhead.\n"
}
```

### 04:58:10 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0004\\hypotheses\\H02.md",
  "content": "# H02\n\nPerforming best-of-2 victim selection on only every eighth eviction rather than on every eviction will capture most of the hit-rate benefit found in cycle/0003/v003 while keeping per-op-time close to cycle/0003/v002.\n\n## Parent\n\ncycle/0003/v002\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, add one field to `TinyLfuCache`:\n\n```rust\nevict_counter: u32,\n```\n\nInitialise it to `0` in `TinyLfuCache::try_new` (add `evict_counter: 0` to the `Self { ... }` struct literal).\n\nIn `TinyLfuCache::put`, at the start of the eviction branch (immediately after the `if self.nodes.len() < self.capacity { ... return; }` early-return block, where `self.head` is the victim candidate), increment the counter and apply best-of-2 selection conditionally:\n\n```rust\nself.evict_counter = self.evict_counter.wrapping_add(1);\nlet victim_idx = if self.evict_counter & 7 == 0 {\n    // Every 8th eviction: compare LRU head vs second-oldest,\n    // evict whichever has lower estimated frequency.\n    let head_idx = self.head.expect(\"TinyLfuCache: full cache has no victim\");\n    match self.nodes[head_idx].next {\n        None => head_idx,\n        Some(second_idx) => {\n            let head_freq = self.sketch.estimate(&self.nodes[head_idx].key);\n            let second_freq = self.sketch.estimate(&self.nodes[second_idx].key);\n            if second_freq < head_freq { second_idx } else { head_idx }\n        }\n    }\n} else {\n    // Remaining 7 out of 8 evictions: plain LRU-head selection (no extra probe).\n    self.head.expect(\"TinyLfuCache: full cache has no victim\")\n};\n```\n\nThen continue with the existing admission-check + replacement code using `victim_idx` (clone the victim key, estimate frequencies, replace if `incoming_freq > victim_freq`). No other changes: the sketch, hash logic, struct fields beyond `evict_counter`, LRU link helpers, and tests remain as in v002.\n\nThe `& 7` mask is equivalent to `% 8` and uses only a bitwise AND, adding no branch prediction pressure beyond the one conditional select. The wrapping add avoids overflow on long-running caches.\n\n## Rationale\n\ncycle/0003/v003 demonstrated that best-of-2 victim selection has a real positive hit-rate signal (+0.0017 over baseline), but the mandatory two extra sketch probes on every full-cache `put` doubled the median eviction latency. Amortising those probes over 8 evictions reduces the average extra-probe overhead to one-eighth of v003's cost. On a stable LIRS working set where eviction frequency (rather than rate) drives hit-rate corrections, even occasional best-of-2 decisions should rescue the most egregious frequency-blind evictions while keeping the common-case path as cheap as v002.\n"
}
```

### 04:58:23 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0004\\hypotheses\\H03.md",
  "content": "# H03\n\nRequiring an incoming item to exceed the victim's estimated frequency by at least two counts before admission will reduce noise-driven eviction churn on the stable LIRS working set and improve hit rate.\n\n## Parent\n\nbaseline\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, change the admission check in `TinyLfuCache::put` — the condition that guards replacing the victim — from:\n\n```rust\nif incoming_freq > victim_freq {\n```\n\nto:\n\n```rust\nif incoming_freq > victim_freq.saturating_add(1) {\n```\n\n`saturating_add(1)` is used rather than plain `victim_freq + 1` to avoid the edge case where `victim_freq == u32::MAX`; in practice counters on this workload will never reach saturation, but the saturating form is correct in all cases.\n\nNo other changes: the struct layout, sketch, hash logic, LRU operations, tests, and all remaining logic in `put` and `get` remain exactly as in the baseline.\n\nThe threshold of 2 (i.e., `incoming_freq > victim_freq + 1`) means:\n- A new item with frequency equal to or one greater than the victim is now **rejected** (previously it was admitted when strictly greater).\n- A new item with frequency at least two greater than the victim is **admitted**, unchanged.\n\nThis tightens admission in the borderline zone where sketch estimation noise is most misleading: two items with near-identical true frequencies may have CMS estimates that differ by ±1 due to hash collisions, causing the cache to flip the victim unnecessarily. Requiring a margin of 2 makes the admission decision robust to single-count estimation noise.\n\n## Rationale\n\nOn the LIRS trace — a stable Zipfian working set — the cache reaches a steady state where most items in the cache are hot and most eviction candidates are cold. The admission check (`incoming_freq > victim_freq`) is correct in most cases, but the marginal case (incoming_freq == victim_freq + 1) is ambiguous: the true frequencies might be equal or even reversed, and the sketch's min-estimate adds noise in both directions. Strengthening the admission condition to require a gap of at least 2 gives incumbents a one-count advantage, stabilising the resident set without blocking genuinely hot newcomers (which exceed any threshold that matters). This tests whether admission stability — an unexplored policy axis across all three prior cycles — can independently lift hit rate on this trace.\n"
}
```

### 04:58:34 chat

All three hypothesis files are written. Here's a summary of what was produced in `cycles/0004/hypotheses/`:

---

**H01 — Two-phase hashing** (`parent: cycle/0003/v002`)  
Replaces v002's single-hash + 4-window scheme with two seeded hashes (seed 0 covers rows 0–1, seed 1 covers rows 2–3), each sliced into two 16-bit windows. This halves hash invocations compared to the original baseline while restoring inter-seed independence between row pairs — closing the tail regression v002 showed on write paths and get p99. **Target: hit-rate and per-op-time.**

**H02 — Sampled best-of-2 eviction** (`parent: cycle/0003/v002`)  
Adds a `evict_counter: u32` field and gates the two-probe best-of-2 victim selection (from v003) to fire only on every 8th eviction (`evict_counter & 7 == 0`). The other 7/8 use the plain LRU-head path inherited from v002. This amortises the sketch probe overhead to one-eighth of v003's cost while retaining most of the hit-rate correction signal. **Target: hit-rate, with controlled latency.**

**H03 — Incumbency margin in admission** (`parent: baseline`)  
Changes the admission guard in `put()` from `incoming_freq > victim_freq` to `incoming_freq > victim_freq.saturating_add(1)`, requiring the newcomer to exceed the victim by at least 2 counts. This makes the decision robust to single-count CMS estimation noise in borderline cases, stabilising the resident set on the LIRS trace's stable Zipfian working set. First test of the admission-threshold axis across all four cycles. **Target: hit-rate.**


---

- finished: 2026-07-20T04:58:35+00:00
- duration: 306.10s
- status: ok
- input_tokens: 15
- output_tokens: 15914
- cache_read_tokens: 453960
- cache_write_tokens: 36230
- cost_usd: 0.512283
