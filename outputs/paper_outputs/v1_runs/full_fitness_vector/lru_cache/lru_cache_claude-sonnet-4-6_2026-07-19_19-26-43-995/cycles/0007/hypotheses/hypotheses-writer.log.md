# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0007\hypotheses
- allow_edits: True
- started: 2026-07-19T18:24:11+00:00

## timeline

### 18:24:15 thinking

Let me read all the context files to understand the current state of the research and then generate 3 distinct hypotheses for cycle 0007.

Let me start by reading the key context files in parallel.

### 18:24:15 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0006\\cycle-insight.md"
}
```

### 18:24:16 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0006\\cycle-winners.json"
}
```

### 18:24:16 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\leaderboard.md"
}
```

### 18:24:17 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cross-cycle-insights.md"
}
```

### 18:24:19 thinking

Now let me read the cycle 0006 variants to understand their implementations, and also read the baseline and some key winning variants.

### 18:24:21 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0006\\variants\\*"
}
```

### 18:24:21 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles-ledger.jsonl"
}
```

### 18:24:24 thinking

Now let me read the v001 overlay from cycle 0006 to understand the current winning implementation.

### 18:24:25 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0006\\variants\\v001\\attempts\\01\\overlay\\src\\lru_cache\\mod.rs"
}
```

### 18:24:26 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline\\src\\lru_cache\\mod.rs"
}
```

### 18:24:29 thinking

Let me find the baseline directory.

### 18:24:29 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline\\**\\*"
}
```

### 18:24:31 thinking

I don't see the baseline src directory. Let me look at it differently.

### 18:24:31 tool: Glob

```json
{
  "pattern": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\**\\*.rs"
}
```

### 18:24:33 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\src\\lru_cache\\mod.rs"
}
```

### 18:24:34 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0006\\variants\\v001\\insight.md"
}
```

### 18:35:12 thinking

Now I have a good understanding of the state of the research. Let me summarize what I know:

1. **Current best variants:**
   - `cycle/0006/v001`: hit-rate=0.7291, metadata-size=16, per-op-time=0 (the cycle 0006 winner, extends the link-array structure with 87.5% protected quota)
   - `cycle/0005/v002`: hit-rate=0.7202, metadata-size=16, per-op-time=0 (the link-array structure without the quota change)
   - `cycle/0002/v001`: hit-rate=0.7202, metadata-size=24, per-op-time=0 (2Q with protected-bit-in-prev, u32 sentinels)

2. **What cycle/0006/v001 does:**
   - Uses u32 sentinel links and separates the link array from nodes so Node<K,V> is just key+value at 16 bytes
   - Implements 2Q policy with the protected-bit stored in the `prev` field of the Link struct
   - Sets a protected quota of 87.5% to control how many items stay in the protected region versus probationary

3. **What has failed:**
   - CLOCK-style deferred relinking, two-hit admission with a seen bit, MRU-tail guards, adaptive quotas, and attempts to compose different winning strategies all haven't panned out across multiple cycles
   - Forced inlining and the entry() API also didn't deliver improvements

4. **Open questions:**
   - Whether pushing the protected quota beyond 87.5% could yield further gains, or if a frequency-sketch-based admission filter might help instead
   - Whether arena allocation or slab grouping could reduce the cache-miss cost of node access

5. **Next steps for cycle 0007:**
   - Use v001's quota policy as the structural foundation, drop the seen bit entirely, then test quota values beyond 87.5% and explore a lighter admission filter based on frequency sketching

Now I'm formulating three distinct hypotheses for the next cycle, making sure at least one builds on a cross-cycle winner—I'm looking at the best performers from cycles 0006, 0005, and 0002 to use as parents for these experiments.

**Hypothesis 1** derives from cycle/0006/v001 and pushes the quota threshold even higher—testing whether protecting 93.75% or 91.7% of capacity yields better hit rates than the current 87.5%, with capacity/16 as the first target since it's a clean power of 2.

**Hypothesis 2** also builds on cycle/0006/v001 but shifts focus to using a compact frequency sketch for deciding when probationary entries graduate to protected status.  The key insight is that a sketch-based approach tracks global access frequency estimates across all accesses, which is fundamentally different from the retired SEEN_BIT that only captured repeat hits within a narrow temporal window on the same slot.  A frequency sketch would carry count information even after an item is evicted and reinserted, creating a "ghost cache" effect without needing actual ghost entries. I'm considering implementing this with a compact fixed-size array of 4-bit counters to keep memory overhead minimal. The LruCache struct size would increase but that wouldn't affect the Node metadata-size metric since the sketch array lives in LruCache, not in Node itself. I'm reconsidering a frequency sketch approach that's genuinely different from the SEEN_BIT mechanism—something that tracks actual access frequency rather than just marking slots as seen, which aligns with the open question about position-based or sketch-based admission variants. The tricky part is deciding what to do when a probationary item gets a hit but its frequency is too low to graduate—I'd need to requeue it at the MRU end of probationary instead of promoting it to protected. A 256-byte Count-Min Sketch array would add minimal overhead to the cache struct while providing reasonable frequency estimates despite hash collisions, and this approach could help filter out items that appear frequently within the window but lack historical access patterns.

Now I'm considering whether to push the quota boundary further or add the frequency sketch as separate hypotheses, both citing the previous cycle's work. I should also think about whether there's a third direction worth exploring around the struct itself. Actually, I'm reconsidering the FIFO approach—it would simplify the protected queue logic by removing the need to reposition items on hits, but that'd likely hurt performance by demoting frequently accessed items too quickly.

Let me explore a different angle: using a frequency sketch to guide demotion decisions. Instead of always evicting the LRU item from protected when it overflows, I could demote based on estimated frequency, keeping high-frequency items protected longer. This targets the demotion path rather than admission, which is a different lever than the SEEN_BIT approach.

Or I could try a finer-grained quota system where the probationary pool has a minimum floor size to ensure it doesn't shrink too aggressively. I'm considering two main hypotheses to test next: pushing to 93.75% protection with capacity/16 (128 slots at capacity=2048), or going even further with capacity/20 or capacity/24 to explore whether there's a diminishing returns curve. The key question is whether the probationary pool has a minimum useful size that creates a floor effect, which would help explain where the performance plateaus.

I'm also exploring some structural alternatives that haven't been tried yet—like a skip-gradient approach where items that hit once in probationary move to the middle rather than graduating directly to protected, creating a softer transition between the two tiers.

But I'm realizing this approach has a fundamental problem: if probationary hits don't promote to protected, frequently accessed items would just accumulate at the MRU end of probationary and never get the protection they need, which would actually hurt hit-rate since the whole point of the protected queue is shielding high-frequency items from eviction when new entries flood in.

I could try an LFU-biased approach instead, where instead of always evicting the LRU item from probationary, I track approximate frequencies and evict the lowest-frequency item among the bottom candidates. That's more complex but might preserve hit-rate better.

Or I could rethink the metadata layout entirely—currently the PROT_BIT is packed into the prev pointer, but I could use separate bit-vectors to track queue membership instead, which would free up bits in the prev field for other purposes.

Looking back at the key open questions: I need to experiment with quota values beyond 87.5% to find where returns diminish, explore a lighter admission filter using a compact frequency sketch, and investigate whether structural improvements like arena allocation or slab-grouping could reduce cache misses in node access.

The current design separates links and nodes into parallel vectors—links pack 8 entries per cache line while nodes only fit 4 per line due to their 16-byte size. I'm wondering if there's a better traversal pattern or interleaving strategy that could improve this. Looking at the hot path in `get()`, it chains together a HashMap lookup, several link accesses to check queue membership and update pointers, and finally a node value read—so the bottleneck seems to be repeated link array accesses across different indices.

The real issue is that when a frequently accessed item is cached, its neighbors in the queue structure are likely cold, forcing additional cache misses. I'm exploring whether we could reduce the number of distinct memory locations touched—maybe through a head-only doubly-linked list with lazy compaction, or by finding optimizations specific to the test capacity of 2048. The most promising angle right now is adjusting the protected quota to 93.75% of capacity and seeing if that shifts the eviction behavior in a useful way.

For the second hypothesis, I'm considering adding a compact frequency sketch—a small fixed-size counter array—to guide which items in the probationary segment get evicted based on their access frequency rather than pure LRU order. This would be distinct from the SEEN_BIT approach since it modifies the eviction decision itself rather than graduation logic, and it accumulates frequency counts across accesses instead of just tracking per-slot bits.

The key insight is that on a Zipfian distribution, high-frequency items naturally graduate to protected quickly, so items lingering at the LRU end of probationary are likely either low-frequency items inserted early or recently demoted items. The frequency sketch would protect those high-frequency items that happen to be at the LRU end due to early insertion, rather than blindly evicting the oldest item regardless of how often it's been accessed. their estimated frequency is low. This is worth exploring: I'd add a frequency sketch to the cache itself (keeping nodes lean) using a compact nibble array for 512 4-bit counters, then increment it on every access. During eviction, instead of blindly removing the oldest probationary item, I'd scan the first few candidates and evict whichever has the lowest sketch count, falling back to FIFO if they're tied. The tradeoff is scanning a small window of items versus the potential benefit of removing genuinely cold entries that were demoted from protected.

I'm drawn to a simpler variant: just peek at whether the probationary head has been accessed more than once according to the sketch, and if so, give it a second chance by moving it back to the MRU end of probationary before trying the next candidate. This mirrors CLOCK's second-chance mechanism but grounds the decision in actual frequency data rather than a per-slot bit. The key difference from the old CLOCK approach is that the work happens only at eviction time, not lazily on every access, so there's no repeated relinking overhead.

For implementation, I could track frequency with a parallel byte vector alongside the nodes and links arrays—one byte per slot for frequency counts. This adds minimal overhead (about 2KB at capacity 2048) without changing the Node struct size, and avoids hash collisions by keying off the arena slot directly rather than the key itself.

But there's a wrinkle: when a slot gets reused for a new item after eviction, the old frequency count would still be there. I'd need to reset the counter to 1 when inserting a new key into a previously-used slot, then increment on subsequent accesses. That way the counter tracks the frequency of the current item occupying that slot, not the slot itself.

This is starting to feel like a variant of the SEEN_BIT approach though. The real distinction would be a global frequency sketch that persists across evictions entirely—tracking access patterns even after items leave the cache.

The idea is that on a Zipfian trace, if an evicted item gets reaccessed, it might be a medium-frequency item that just fell out of the hot window temporarily. A persistent sketch would recognize it's been seen before and prioritize it accordingly. This is similar to ARC's ghost entries, but using a bloom filter or count-min sketch instead of maintaining actual ghost entries.

I'm thinking of adding a count-min sketch structure to the cache that tracks estimated frequencies across the entire history, with multiple hash functions and counters to estimate how often each key has been accessed.

The key insight is that a frequency sketch can distinguish between items based on their total access count across eviction boundaries, whereas the SEEN_BIT approach only tracks whether an item was accessed during its current probationary period. This unlimited window is what gives the frequency sketch its advantage.

However, a sketch alone doesn't capture recency—an item accessed frequently in the past but cold now would still rank high. The most practical approach would be to use a global fixed-size sketch that increments on every access, then use the sketch count to inform eviction decisions within the probationary tier by preferring to evict items with lower frequency estimates rather than strictly following LRU order.

Now I'm considering a completely different direction: the SIEVE policy from recent research. It's elegantly simple—items carry a single visited bit that gets set on access, and eviction scans from a hand position to find the first unvisited item, resetting visited bits as it goes. This avoids the reordering overhead of LRU while performing well on skewed access patterns, and it only costs one bit per node.

But I'm second-guessing whether SIEVE is actually distinct from CLOCK, which was already rejected. The core difference seems to be that SIEVE doesn't relink items on hits—it just sets a bit and lets the hand sweep backward through the list on eviction, whereas CLOCK uses a circular buffer with explicit relinking. Still, the fundamental mechanism of second-chance eviction with a scanning hand feels similar enough that it might face the same objections.

What's compelling though is that in our link-based setup, we could implement SIEVE without the expensive unlink-and-push-back operations that plague every get() call right now. We'd just toggle a visited bit and let the eviction scan handle the rest.

The cache-line pollution concern from CLOCK still applies here—the hand sweeping through the linked list during eviction could touch many cache lines, especially if most hot items are marked visited and we need to scan far to find an unvisited victim. But unlike CLOCK's circular buffer, SIEVE uses a true linked list, so the sweep is bounded by the actual items we traverse rather than a fixed buffer size.

Looking at SIEVE's actual mechanism: it maintains a FIFO queue with a hand pointer that marks items as visited on access without moving them, then during eviction it scans backward through the queue, resetting visited flags to zero and advancing the hand until it finds an unvisited item to evict.

The key difference from CLOCK is structural—SIEVE uses a true queue where the hand moves backward from the LRU end toward the MRU end, whereas CLOCK uses a circular array with forward scanning. But the fundamental issue remains: on a cache dominated by hot items (95%+ hit rate), eviction still requires scanning through many visited items before finding a candidate, making the operation potentially expensive.

The advantage is that get() stays O(1) with no movement overhead. evictions are rare enough that SIEVE's main benefit—avoiding relinking during cache hits—could outweigh the cost of scanning during evictions. The scan length depends on how many items are marked as visited, which on a Zipfian distribution could get expensive if most hot items are already marked. It's a promising direction but carries some risk, so I'll present it as a more conservative alternative to the baseline rather than pushing it as a primary recommendation.

For the quota exploration, I'm considering testing capacity/16 as my first hypothesis at this capacity level—that would protect 93.75% of the cache. If it performs well, the next cycle can push to capacity/32; if it hurts, I'll know that 87.5% is close to optimal. I could test two quota values simultaneously, but that would consume two hypothesis slots and leave less room for other novel ideas. Since I need three distinct hypotheses with at least one referencing a cross-cycle winner, I should be strategic about which quota value to prioritize. I'm sketching out a frequency tracking approach using a simple array of counters, but I'm realizing the hash collision problem—multiple keys mapping to the same counter would pollute the frequency estimates for unrelated keys. I'm considering switching to a Count-Min sketch with multiple hash functions instead to get more reliable frequency estimates for eviction decisions. Implementing a top-K scan in evict_one() would require traversing the probationary list to compare frequency estimates and unlinking the lowest-frequency candidate, which adds overhead but could improve eviction decisions on Zipfian workloads where most evictions target low-frequency items anyway.

I'm reconsidering this approach and exploring an alternative: reordering the probationary queue based on sketch-derived frequency estimates when items are inserted or demoted. The idea is that when a demoted item (which likely has higher frequency) enters probationary at the MRU tail, it should be positioned differently than a fresh item to reflect their relative eviction priority—protecting the demoted item while making the never-before-seen item a better eviction candidate.

But this creates tension with 2Q's core design: the FIFO ordering of probationary is intentional, and a frequency sketch might contradict it by suggesting that an old, rarely-accessed item should be kept while a recently-demoted, previously-hot item should be evicted first. I'm weighing whether this hybrid approach actually improves the cache or just muddies the original 2Q philosophy.

The simplest path forward is to add a small Count-Min Sketch (around 512 entries) that tracks access frequency across all operations, then during eviction check the two candidates at the head of probationary and evict whichever has the lower frequency estimate. This gives us a lightweight "2-choice eviction" strategy that should perform better on Zipfian workloads without major complexity.

Implementing this sketch... functions. This avoids the overhead of a full hash function and keeps the bit operations cheap, which is crucial since we're calling this on every get/put operation.

For the H02 implementation, I'm building a tiny count-min sketch using four rows of 64 byte counters—just 256 bytes total—that saturates at 255 to estimate key frequencies for eviction ranking.

The challenge is that K is generic, not necessarily u64, so I'd need to hash it first through the Hasher, which adds overhead. Since the HashMap already has K's hash internally, I could either extract it by calling the hasher ourselves, or store the hash value directly in each Node alongside the key and value.

But storing the hash grows the Node struct from 16 bytes back to 24 bytes for u64 keys—that's a step backward. Alternatively, I could track the slot index's hash instead of the key's hash, but that would conflate different keys occupying the same slot over time, making the frequency estimate useless.

Let me step back and try a simpler approach: instead of a frequency sketch, I could use a 2-random-choice eviction policy where I pick two random candidates from the probationary LRU end and evict the older one.

The problem is determining "value" without a sketch. I could rank by position in the LRU order, but that just means always evicting the head—which is what I'm already doing. Without a frequency estimate, random-choice collapses into FIFO.

So let me refocus on three concrete hypotheses: first, extend the protected quota to 93.75% as a pure quota experiment; second, test an even more aggressive quota at 96.875%.

But wait—these are nearly identical, just different quota values. The quality bar requires distinct hypotheses, and near-duplicates collapse the cohort. I should use one quota experiment and two structurally different ideas instead.

For the third hypothesis, I could try SIEVE-style eviction to eliminate the relinking cost on get() operations. This is genuinely novel—not yet tested in this research—with the claim that avoiding relinking on hits improves per-op-time while maintaining or improving hit-rate. Though SIEVE was conceptually retired as CLOCK, the core insight still applies.

The key distinction is that SIEVE addresses the specific objection about hand-sweep cache pollution: no sweep happens on hits, only on eviction, and the hand naturally scans the linked list in order. Looking at the structural differences more carefully—SIEVE doesn't relink on hits (just sets a bit), while CLOCK keeps items in place but sweeps from a circular position. The hand in SIEVE scans backward from the current position toward recently inserted items, whereas CLOCK scans forward continuously. This gives recently inserted items natural protection since they're at the MRU end and get swept last.

I need to clarify the direction: new items go to the head (MRU end), the hand starts at the tail (LRU end), and on eviction it walks toward the head, skipping visited items and resetting their bits until finding an unvisited candidate to evict. The concern about cache-line pollution comes from the hand touching many cache lines while scanning through visited items before finding an eviction target, which becomes significant at high fill factors like 73%.

Now I'm realizing SIEVE doesn't have the protected/probationary distinction that 2Q provides—it's just a flat list with a visited bit. That means SIEVE might actually be weaker than our current 2Q design since 2Q explicitly shields frequently-accessed items in the protected queue, whereas SIEVE only approximates protection through the visited bit without distinguishing frequency levels.

What if I combined them? I could keep the protected queue as-is with LRU discipline and relinking on hits, but apply SIEVE-style eviction to the probationary queue instead—no relinking on hits there, just setting a visited bit, and scanning from the hand position until finding an unvisited item to evict. This hybrid approach would preserve the protected queue's LRU guarantees while reducing relinking overhead in the probationary queue.

But there's a deeper issue: if probationary hits just set the visited bit without graduating to protected, items would stay in probationary much longer, and the protected queue would gradually drain as items age out. Eventually most items would end up in the flat SIEVE pool, which defeats the whole point of having a protected tier.

I could add both mechanisms—set visited=1 AND schedule graduation on a probationary hit—so items get a second chance during eviction if they were recently accessed. But this adds complexity and it's unclear if it's actually better.

Let me step back and try a cleaner approach: replace the entire 2Q structure with a single SIEVE-managed list. This gives me a clean baseline to compare against the pure LRU baseline.

The implementation uses a parallel visited array alongside the nodes, with a hand pointer tracking the eviction position. On access, I just flip the visited bit without any relinking—that's the key efficiency win. When evicting, I advance the hand through the list, clearing visited bits until I find an unvisited node to remove. icts an item and a new item is inserted at the tail. So the hand only moves toward the head when evicting, and wraps back to the tail once it reaches the head. Newly inserted items start with visited=false, hot items get visited=true and survive the sweep, and the hand gradually works through older items in the queue.

Actually, this is just CLOCK implemented with a linked list instead of a circular array—the core algorithm is identical. The paper's main contribution is demonstrating it performs better in practice, but CLOCK-style approaches have already been dismissed in the literature.

Let me move away from SIEVE entirely and explore something genuinely different. I'm considering a two-tier probationary queue approach where newly inserted items go into one queue and items that were previously evicted or demoted go into a separate queue, with eviction prioritizing the newer probationary tier first.

To distinguish between truly new items and those that have been in the cache before, I'd need to track evicted keys somehow—either through a ghost list that stores recently evicted keys, or a more space-efficient bloom filter. A ghost list adds overhead but is straightforward, while a bloom filter trades some accuracy for lower memory cost. This is essentially the ARC policy's approach.

The bloom filter idea has a catch though: it doesn't support deletion, so after many evictions the filter saturates and loses its ability to distinguish new from repeat accesses. A counting bloom filter or periodic reset could fix this, but there's a simpler alternative—just using a small hash set to track recent evictions.

Actually, I'm overthinking this. Let me focus on hypotheses that directly test what the cycle insights suggest.

I'm settling on two concrete experiments: first, pushing the protected cache quota up to 93.75% to see if giving more space to hot items improves hit rate, and second, testing lazy demotion where we avoid pushing items out of protected when it's only slightly over quota. The current implementation stores links and nodes separately, so accessing a hot item requires reading from both arrays. If I reorganize the arena periodically to cluster protected items together by index, both reads would hit the same cache line more often. But this would require updating all the HashMap pointers and link fields, which is O(n) work that could cause latency spikes—probably not worth the complexity given the performance constraints.

Another angle: I could add software prefetch hints when accessing nodes during get() to hide memory latency while processing links. The issue is that Rust's stable API doesn't expose prefetch instructions directly—they're only available through unstable intrinsics, which would require unsafe code.

However, `core::arch::x86_64::_mm_prefetch` is available in stable Rust on x86_64 platforms, so I could use that to prefetch the node data while the HashMap lookup is still in flight. This would be a targeted optimization on the hot path without requiring external dependencies.

The idea is to issue a prefetch instruction for `nodes[idx]` right after the HashMap lookup but before we actually need to read from it, which would hide the latency of bringing that cache line in from memory. Since `links` and `nodes` are separate vectors at different memory locations, the node data might not be in cache when we first access it, making this prefetch worthwhile on repeated accesses.

That said, modern CPUs with out-of-order execution and hardware prefetchers might already be smart enough to handle this pattern automatically, and the measurement granularity (100ns per cache tier) could make subtle improvements hard to detect. Still, it's a straightforward hypothesis to test—just add a platform-specific prefetch call in the hot paths without changing any logic. But I'm skeptical this falls into the micro-optimization category that hasn't paid off before.

Now I'm thinking about a different angle: what if the real issue is the HashMap's load factor? If I pre-allocate with extra capacity—say, `capacity × 9/8`—I could reduce the load factor from around 87.5% to 77.8%, which would shorten hash probe sequences. The tricky part is understanding exactly how `with_capacity` reserves space and maintains load factor at steady state.

Actually, that's probably not worth pursuing since `with_capacity` already handles the allocation reasonably well. Instead, I should test some intermediate values for the probationary quota—something like `capacity/12` would sit between `capacity/8` and `capacity/16` to narrow down the sweet spot. But that feels redundant with what I'm already exploring.

Let me stop second-guessing myself and commit to three concrete hypotheses: first, pushing the protected quota to 93.75% by setting `prob_cap = capacity/16`; second, testing a different approach; and third, another variant to compare against.

The second hypothesis involves adding a compact count-min sketch to track item frequencies in the probationary section, using just 256 bytes of extra memory to make smarter eviction decisions by preferring to evict rarer items.

The key insight is that the sketch accumulates frequency across an item's entire history, not just since insertion, and we only need to update it during evictions—not on every get/put operation. But to have meaningful frequency estimates, we'd need to increment the sketch whenever we access an existing key, which adds per-operation overhead that could hurt performance.

For the hash functions, since keys are u64, we could extract bits directly as indices into the sketch rows. But the eviction function is generic over K, so we'd need to rely on the Hash trait instead of assuming u64 keys.

The overhead concerns me: creating a DefaultHasher, hashing the key, finishing to get a u64, computing four indices, and incrementing counters adds maybe 10-20 instructions per get or put-of-existing. On a tight budget where operations should be under 100ns, that could push latency noticeably. I'm wondering if there's a way to avoid this per-operation cost.

One option is storing the hash alongside the key in each node—adding a u64 field would make Node 24 bytes instead of smaller, but that grows the metadata overhead. That's not appealing. Let me step back and think simpler.

What if I just compare two candidates in eviction without tracking frequency—say, prob_head and the next item? But since it's FIFO, prob_head is always older, so I'd always evict it anyway. That doesn't actually help. The two-choice idea needs some ranking mechanism to pick between candidates.

Actually, I could store a hash fingerprint in the Link struct itself. If I add a u16 field to Link to hold the lower 16 bits of the key's hash, it grows from 8 to 12 bytes (with padding), but that only affects the links Vec size, not the Node metadata score. During eviction, I could use this fingerprint for a lightweight priority decision without maintaining a full frequency sketch.

But let me keep this simpler. I'll add a separate Vec<u8> for hit counts—one byte per slot, reset when a slot gets reused for a new key, and incremented on access. During eviction, I compare the hit counts of the two candidates and evict the one with fewer hits. The per-operation cost is just a single saturating increment, and the eviction path adds one extra read and comparison.

Actually, I'm realizing there's a subtle issue here: in 2Q, a hit on a probationary item immediately graduates it to protected. So probationary items shouldn't accumulate many hits—they'd be promoted away. The only way an item ends up back in probationary is if it gets demoted from protected when that region overflows. When that happens, its hit count carries over from its time in protected, but since I'm resetting hit counts on eviction, not on demotion, the count would persist.

This creates a problem: a demoted item with a high hit count (say 5) would be kept over a freshly-inserted item with hit count 0, even though the demoted item was cold enough to get evicted from protected in the first place. That's backwards—the new item might be the next hot one, while the demoted item is clearly not worth keeping.

The real issue is that hit_count reflects past frequency, not future frequency. On a Zipfian trace, if an item's frequency dropped enough to push it to the LRU of protected, we shouldn't preserve its old hit count when demoting it. Resetting hit_count to 0 on demotion (rather than only on eviction) would make probationary hit_count reflect only recent activity since entering that tier. Though in the current 2Q design, probationary items graduate immediately on any hit, so they'd have hit_count at most 1 anyway.

I'm probably overthinking the hit_count angle. The real constraint is that probationary items are designed to have 0 or 1 hits—anything with 1 hit gets promoted—so hit_count can't meaningfully rank eviction candidates there.

What if instead I sidestep the hit logic entirely and focus on reducing per-operation time? The cross-cycle notes mention that arena allocation or slab grouping can reduce node-access cache-miss costs without adding per-op branches. I'm considering whether aligning the Vec allocations to cache-line boundaries would help, since Rust's default allocator may not guarantee 64-byte alignment.

Currently, Link is 8 bytes with 4-byte alignment, so 8 links fit in one cache line but the Vec might not start on a cache-line boundary. Node is 8 bytes with 8-byte alignment. If I force 64-byte alignment for both Vecs, reads would map cleanly to cache lines—but the stdlib allocator probably already handles this reasonably well.

I could use a custom allocator or pad structures to 64-byte alignment, like creating a LinkLine struct that holds exactly 8 links. However, this adds complexity. The real question is whether alignment is actually causing cache misses—on modern CPUs, as long as a single struct doesn't straddle two cache lines, it's fine. Since Link is only 8 bytes, it never straddles a 64-byte cache line regardless of its 4-byte offset, so alignment isn't actually a problem here.

What about interleaving links and values instead? Right now links and nodes are in separate vectors, so accessing both for a `get(idx)` operation means jumping between distant memory locations. I could combine them into a single struct that holds the link pointers alongside the key-value pair, which would improve locality. This would grow the per-slot size though—for K=u64, V=u64, a combined slot would be 24 bytes instead of separate allocations.

The tradeoff is interesting: we'd lose some space efficiency in the metadata dimension, but gain cache locality since the link pointers and actual data would sit in the same cache line. The measurement framework might still report the Node size based on whatever the current Node type definition is in the overlay.

I think I should just focus on testing a few concrete hypotheses instead of getting too deep into the layout optimization. First, I could push the protected quota up to 93.75% to see if that threshold matters. Second, I could try merging the link fields directly into the Node struct to test whether the cache locality improvement outweighs the metadata size increase.

But wait—that second approach would actually regress us back to the cycle/0002/v001 design with 24-byte metadata, which we've already moved away from. The previous experiments showed that splitting the links into a separate array was cost-neutral, so the cache locality argument probably isn't as strong as it seems. I should skip that option and explore something different instead.

Let me reconsider H02 and H03 with fresh ideas. Testing quota at capacity/12 would give us another data point between our known capacity/8 result and the unknown capacity/16, but it's too close to H01 to be useful. And the biased protected eviction idea doesn't actually make sense in this context.

I'll stick with my three strongest candidates, even though two of them branch from the same parent cycle/0006/v001.

For H01, I'm pushing the quota to 93.75% with a smaller probationary region to see if that improves hit-rate. H02 is adding a compact frequency sketch to score eviction candidates in the probationary section—this is distinct from the SEEN_BIT approach since it changes how we select victims rather than when we graduate entries. H03 switches to SIEVE eviction from the baseline to test whether per-operation time improves, since SIEVE avoids the relinking overhead that CLOCK has. I'm framing SIEVE carefully to address the earlier CLOCK rejection: it uses a linked-list structure instead of a circular array, so there's no fixed-size overhead, and the hand sweep only happens at eviction time rather than on every access.

The real advantage is that on a 73% hit-rate trace, most operations are cache hits that don't trigger any sweep at all—only the 27% of misses that cause insertions need to run the eviction logic. And crucially, get() in SIEVE just sets a bit with zero pointer manipulation, which is much simpler than the current LRU/2Q hot path. But I'm aware the cycle feedback said CLOCK-style deferred relinking has failed twice, and technically SIEVE uses the same mark-on-access, sweep-on-eviction pattern, so the researchers might see it as too similar to revisit.

To frame H03 as truly distinct, I should emphasize the position-stable slots with lazy recency markers—no fixed-size array like CLOCK had, no hand that continuously sweeps on every eviction. Instead, the hand in SIEVE is bounded and scans from where it left off, and on a Zipfian trace with high hit rates, most items near the LRU end already have their visited bit cleared from previous sweeps, so the hand rarely needs to scan far. In steady state on a high-hit-rate trace, the hot set dominates, and when eviction is needed, the scan from the current hand position is efficient. Reducing the probationary pool further from 12.5% to 6.25% should keep more items in the protected queue and boost hit rates. I'm also considering adding a compact count-min sketch to the cache structure to track item frequency estimates, which would help identify better eviction candidates when the LRU end doesn't have cold items available.

But there's a real concern here: updating the sketch on every access adds overhead since each get or put of an existing key would need to hash the key and update four counters. That could push operation time from negligible into the 100ns range, which might outweigh the hit-rate gains. I'm trying to figure out if there's a way to limit this overhead without sacrificing the frequency tracking benefits.

What if I only maintain the sketch on insertions rather than every access? That would give me a signal about whether a key is returning (evicted and reinserted) versus brand new, which might be more reliable than tracking raw access frequency.

On a Zipfian distribution, the hottest keys get inserted once and never evicted, so their sketch count stays at 1. Keys that get evicted and come back have a count of 2 or higher. When I need to choose between two probationary candidates, I can prefer the one with a higher sketch count since it's proven to have recurring demand. This is essentially the frequency filter from Caffeine's TinyLFU design. The overhead drops significantly too—I'm only updating the sketch on new insertions, not on every hit.

For the revised H02 approach, I'm adding a 256-byte sketch array to the cache struct and incrementing it whenever a key first enters or re-enters the cache. Then during eviction, I check the sketch counts to bias toward removing one-hit-wonder keys instead of keys with proven recurring demand.

The sketch uses a simple hash-based approach to map keys to 4 different counters across 4 rows, which keeps the memory footprint small while still capturing frequency information. Since the sketch updates only happen during insertions rather than on every access, there's no overhead to the hot path.

With 2048 keys distributed across 64 slots per row, we'd expect around 32 keys per slot on average, but a Zipfian distribution means popular keys cluster in certain slots while others remain sparse. The minimum-of-4-rows estimate naturally favors popular keys since they're less likely to have all 4 hash positions collide with other high-frequency keys. For eviction purposes, a one-hit-wonder would show a sketch count around 1, while a returning key would have count ≥2, making the distinction clear enough in practice.

The overhead is minimal—just one hash operation per insertion and a few array accesses per eviction. The real question is whether this actually improves hit-rate: if the LRU head of the probationary queue happens to be a returning key while newer insertions are one-hit-wonders, we'd flip the eviction order and keep the returning key instead.

On a Zipfian trace, cold items naturally accumulate at the LRU end of probationary, so the current FIFO eviction already tends to remove the right things. The sketch would mainly help when the LRU head is a demoted-from-protected item that was previously hot but is cooling down, and we need to distinguish between that and a brand-new item that's never been seen before. The mid-tier item might get another access and graduate to protected, while the cold item gets evicted before polluting the cache. But this scenario only helps when a warm item happens to be at the LRU head with a colder item behind it—on a Zipfian trace with a large protected pool, the probationary head is mostly old cold insertions rather than recently demoted items, so the sketch would rarely encounter this situation.

I'm stepping back to focus on the strongest ideas: pushing the quota beyond 87.5%, exploring frequency sketches despite their complexity, or trying a completely different structural approach. What if instead of immediately reusing the evicted slot, I use a free-list for slot management? This would decouple eviction from reuse—the new key wouldn't necessarily take the same memory location as the evicted item.

The tradeoff is that hot items could end up scattered across memory rather than clustered together, which might hurt cache locality. A bump-allocator approach could pack newly-inserted items into lower indices, but that adds complexity without obvious gains.

So I'm narrowing down to three concrete hypotheses to test: increasing the quota to 93.75%, skipping relinking for protected queue hits when the item is already in the top half, and one more variant to measure the impact.

For the lazy promotion idea, when a protected item gets hit, I'd skip the unlink and push_back operation if it's already positioned well enough—specifically if it's already at the MRU tail, since that means it was just accessed and doesn't need to move.

I could add a short-circuit check: if the item's next pointer is NULL and it's in the protected queue, we skip the relinking entirely since it's already in the optimal position. But this only catches the single most recently accessed item, so it's a narrow optimization.

A broader approach would be to skip relinking whenever the item's next pointer already points to the current protected tail, which would catch more cases where the item is already positioned near the MRU end.

I'm overthinking this though. The lazy promotion idea with generation counters adds too much complexity. Let me settle on a cleaner approach: expand the protected quota to 93.75% of capacity, which is a one-line change in the initialization code.

For the second option, I'm considering a small count-min sketch to score eviction candidates in the probationary queue, using only insertion events to update it rather than every access. This keeps the hot path clean since updates only happen during puts for new keys. But I'm running into a snag—the sketch estimation needs access to the node's key to look up the estimate.

In the eviction logic, I'd grab the first two probationary nodes, hash their keys through the sketch, compare the estimates, and evict whichever has the lower score. Reading two keys from the nodes array could trigger a cache miss or two depending on alignment, but that's acceptable since eviction itself is already an expensive operation. The hashing overhead is unavoidable for a generic key type, but it's a constant cost.

This feels like a solid approach worth pursuing. For H03, I'm considering what genuinely novel hypothesis I could test beyond the baseline. The cross-cycle notes mention key-clone elision, but that needs a heap-owning key type which the benchmark doesn't use. I could test 2Q with an even higher protected quota—pushing prob_cap to capacity/32 for 96.875% protection—but that's too similar to H01. Let me explore something different, like enforcing strictly sized queues through probing.

Actually, here's a better angle: the cycle insights show that all two-hit mechanisms have failed because graduation to protected requires a second access. What if instead of waiting for a second hit, I used a time-to-live mechanism where items in probationary get automatically promoted to protected after spending X operations there? This shifts from merit-based to seniority-based promotion, avoiding the strict two-consecutive-access requirement. I'd track an operation counter in the cache and record when each probationary item was inserted, then promote the oldest one to protected when eviction is needed if it's exceeded the TTL threshold.

The challenge is storing insertion timestamps per item—either expanding the Node struct or maintaining a parallel vector of timestamps, both adding memory overhead. Even using a u32 counter instead of u64 still requires auxiliary storage. And the cycle insights suggest that time-adaptive mechanisms might be wasted on a stationary distribution anyway.

Let me pivot to a different structural approach instead. Going back to cycle/0002/v001 as a parent doesn't help since cycle/0006/v001 already dominates it with better metadata-size and similar hit-rate. I need to explore genuinely untested approaches instead of retreading the same optimization paths.

Looking at what's been explored across all cycles—sentinel links, 2Q policies, MRU guards, quota adjustments, link-array splits, SEEN_BIT filtering, adaptive quotas, and CLOCK-style eviction—versus what hasn't: SIEVE, ghost lists tracking recently evicted keys, and sketch-based eviction. The sketch approach for H02 seems like the most promising direction forward.

For H03, I'm thinking about using a compact bloom filter embedded in the cache struct to remember which keys were recently evicted, then fast-track any re-insertions of those keys directly into the protected queue instead of starting them in probationary. I'd add a 512-bit bloom filter using three hash functions to mark evicted keys, and check it during insertion to decide queue placement.

The idea is that on Zipfian workloads, items that cycle back in are likely genuinely popular since they've already proven their value. But bloom filters have false positives—unrelated keys might collide and get incorrectly promoted—and the filter saturates over time without clearing. A counting bloom filter with nibble counters would let me decrement on reinsertion and increment on eviction, enabling proper deletion semantics.

A simpler alternative: maintain a small fixed-size hash set of recently evicted key hashes (say, 64 slots). When a new key arrives, check if its hash matches any in the set—if so, insert directly to protected. When evicting, add the key's hash to the set, pushing out the oldest entry. The lookup is either a linear scan of 64 entries (fast, cache-friendly) or a power-of-2 hash table for true O(1) access, though this adds complexity with key comparisons when promoting from probationary to protected.

The main concern is false positives: a new key that happens to collide with an evicted key's hash gets incorrectly sent to protected, potentially displacing hot items. On Zipfian traces, this is especially risky—cold items that reappear get logged and protected, crowding out the truly hot items and degrading hit-rate. With a 87.5% protected quota, this misallocation becomes more costly.

An alternative approach is to split the probationary queue into two sub-queues: one for new insertions and another for items demoted from protected. This follows the 2Q+ variant, where demoted items are treated differently and can be re-promoted more easily. It adds complexity, but might better distinguish between first-time accesses and items that have already been in the protected tier. Adding queue variants...

Actually, I'm realizing the original 2Q algorithm is more nuanced than what I was sketching. It uses three queues: Ain for first-time accesses (FIFO), Aout as a ghost list tracking recently evicted pages from Ain, and Am as an LRU queue for pages that have been accessed multiple times. When a page is first accessed it goes into Ain, and if Ain fills up, the tail gets evicted to Aout. If something in Aout gets re-accessed, it graduates to Am instead. The Am queue handles the popular items with LRU semantics.

The version we're actually building skips the ghost list entirely—any probationary hit immediately promotes to protected. But a more faithful implementation would track recently evicted probationary keys in a small ghost set, so returning items can be promoted directly to protected while new items start in probationary. This is essentially what I was thinking with the eviction bloom filter, except using exact tracking via a hash set instead of probabilistic filtering.

So the candidate approach is to add a small ghost list of 32-64 entries for recently evicted probationary items, letting returning items skip straight to protected.

Adding ghost list overhead... The linear scan through 128 entries is actually quite fast in practice—even with scalar operations it's under 10 nanoseconds. Since the key is already being cloned during eviction for Copy types like u64, storing it in a ghost list is essentially free. I'm thinking of implementing this as a ring buffer with a write pointer, where recently evicted probationary keys get stored and checked on insertion attempts.

Defining the ghost list structure... A linear scan through 64 cache lines would be prohibitively expensive, so I'd need to either use a HashSet for O(1) lookups (though that adds overhead) or keep the ghost list very small—maybe 32-64 entries to stay within a few cache lines. The tradeoff is that a tiny ghost list would catch very few returning items at a 2048 capacity, which undermines the whole purpose of the ghost list.

Let me step back and focus on the clearest hypotheses without overcomplicating this. The strongest candidates are pushing the quota to 93.75%, implementing a compact CMS for eviction ranking, or trying a small ghost list for returning items. There's also SIEVE as a higher-risk alternative if the other approaches don't pan out.

Now I'm realizing the requirement is just that at least one hypothesis needs to cite a cross-cycle winner, not all of them. So I can diversify the parent cycles across my three main hypotheses rather than clustering them all under the same one, which would make them feel like near-duplicates. no 2Q complexity) - tests whether skipping relinking on cache hits improves per-operation time. For the SIEVE implementation, I'm starting from the baseline LRU structure and adding a visited bitmap alongside the nodes, plus a hand pointer for the eviction candidate. On cache hits, I just mark the entry as visited instead of relinking it, and the same applies to updates—no queue manipulation needed.

For new insertions, I add the entry at the tail (most recently used position) with visited set to false, keeping the hand unchanged. During eviction, the hand scans backward toward the head, clearing visited bits on items it encounters; once it finds an unvisited entry, that's the victim. I'm clarifying the hand direction here—it moves from the tail toward the head (oldest items), and rather than wrapping around, it continues scanning from its current position until finding something to evict.

Now I'm working through the concrete implementation details: the list structure has a head pointer for the LRU end and a tail for the MRU end, with a hand position that tracks where we are in the eviction scan and a visited array tracking which items have been accessed. On get or put operations for existing keys, I mark the item as visited without relinking it in the list. For new insertions, I need to handle the arena allocation and insertion logic.

The eviction process scans from the current hand position toward the head, skipping over visited items by clearing their flags and advancing the hand, until finding an unvisited item to evict. Once evicted, the hand moves to the previous position. When the hand reaches the head, the scan wraps back to the tail to continue the sweep.

For the Node structure, I'm checking the memory footprint—a baseline node with key, value, and two Option<usize> pointers should be 32 bytes for u64 keys and values, but I need to verify this against the fitness dimension requirements. V=u64, the size depends on how Rust optimizes `Option<usize>`. Since `usize` can legitimately be 0, the compiler can't use niche optimization, so `Option<usize>` would typically be 16 bytes rather than 8. That would make the node 8+8+16+16 = 48 bytes, which matches the baseline measurement.

The link-array optimization in later cycles extracted the prev and next pointers into a separate structure, shrinking the node itself to just 16 bytes. Now I'm calculating what happens when we add a `visited: bool` field to support a SIEVE cache implementation—that would push the node size beyond the current baseline.

To keep the node compact, I should store the visited state in a parallel `Vec<bool>` instead, maintaining the 48-byte baseline. But I'm realizing the baseline itself already assumes the link-array split has been applied, so the node size calculation needs to account for that architecture.

Looking at the scoring, cycle/0006/v001 achieves metadata-size=16 with per-op-time=0, which heavily outweighs the baseline's 48-byte overhead. For a SIEVE implementation starting from baseline to win, it would need to dramatically improve hit-rate and per-op-time to compensate for the metadata-size penalty.

But what if I layer SIEVE on top of the link-array structure instead? The Node stays at 16 bytes, and I'd add a parallel visited vector (1 byte per slot) plus a hand pointer in the cache struct. The implementation would use a single queue without the probability/protection split, simplifying the link structure to just prev/next pointers. On get and put operations, I'd simply mark visited without relinking, only performing link operations on new insertions.

The eviction process would advance the hand pointer from the tail toward the head, clearing visited bits until finding an unvisited entry to evict. This keeps the same 16-byte node size and metadata footprint as the current winner, with potentially faster per-operation time since gets avoid relinking entirely. The tradeoff is losing the two-queue policy—hit rates might drop back to baseline LRU levels or wherever SIEVE performs on this workload, whereas the cross-cycle data suggests exact LRU ordering consistently beats coarser recency signals like single-bit CLOCK or two-hit gates.

I'm framing this as a clean experiment: SIEVE eviction on the link-array structure trades 2Q precision for zero per-hit relinking cost. Starting from the current winner would mean stripping out the prob/prot queue heads, tails, and length tracking.

The risk is real—SIEVE might underperform if the workload doesn't suit its coarser recency signal. But SIEVE has outperformed LRU on many traces, and unlike CLOCK's deterministic cycling, SIEVE gives every item a second chance as the hand sweeps and clears visited bits, so recently-accessed items get another window before eviction.

The real question is whether SIEVE's tail-to-head scan direction matters for Zipfian workloads compared to CLOCK's head-to-tail approach. I should propose SIEVE as a fresh baseline variant and accept that the fitness score might penalize it heavily due to the 48-byte node size, but that's worth testing to see if the policy itself is stronger.

### 18:44:05 thinking

Let me now synthesize the hypotheses based on what I've read. Here's what I know:

**Current state:**
- Best winner: cycle/0006/v001 — hit-rate=0.7291, metadata-size=16, per-op-time=0
- This uses: link-array split (Node=16 bytes), 2Q policy with prob_cap = capacity/8 (87.5% protected quota)
- SEEN_BIT two-hit admission is retired — conclusively failed on workload-a
- All two-hit mechanisms failed across 3 cycles

**Open questions from cross-cycle insights:**
The main unknowns are whether pushing the quota beyond 87.5% unlocks more gains, whether a lighter admission filter like a frequency sketch could work better than the failed two-hit approach, and whether memory layout optimizations could reduce cache misses on node access. Based on the cycle-0006 insights, I should test higher quota values, explore a compact frequency sketch as an alternative to SEEN_BIT's strictness, and avoid revisiting two-hit mechanisms. Any new hypotheses need to build on one of the proven winners and represent genuinely distinct strategies.

For H01, I'm testing whether reducing the probationary quota from 12.5% to 6.25% (pushing protected quota to 93.75%) continues improving hit-rate or reveals a reversal point—this directly extends cycle/0006/v001's direction by adjusting the capacity divisor in the quota calculation.

For H02, instead of relying on a two-hit SEEN_BIT pattern, I'm adding a compact frequency sketch—a small 4-row CountMin structure with roughly 64 counters backed by a byte array—that tracks approximate access frequency across the entire cache lifetime. Items graduate from probationary status when their estimated frequency exceeds 1, which better captures genuinely frequent items on Zipfian traces since most items have frequency 1 anyway. This approach survives eviction cycles unlike SEEN_BIT, which only counts within a local window. The key insight is that a frequency sketch tracks accesses across the entire lifetime, including cache misses, so when an item gets its first probationary hit, I can check if the sketch count is already ≥ 2 — meaning it was accessed before insertion or after eviction. If so, I graduate it immediately to protected, avoiding starvation of the protected pool. This way items with genuine popularity across the trace get promoted on their first cache hit rather than waiting for a second sequential access within the probationary window.

For the implementation, I'd increment the sketch in `put()` before checking if the key exists, then in `promote()` check if the sketch count exceeds 1 to decide on graduation. I'm thinking about using a 4-row CountMin sketch with 16 buckets per row — that's 64 bytes total, fitting in a single cache line — and I'll need to hash the key with multiple independent seeds to populate each row.

Now I'm looking at slab grouping and cache-line alignment. The current layout stores nodes in a Vec and links separately, with nodes being 16 bytes each (4 per cache line) and links at 8 bytes each. The access pattern chains through HashMap lookup to get an index, then accesses the links array, which creates multiple memory round-trips.

The challenge is that without knowing access patterns ahead of time, spatial grouping is difficult. Approaches like always inserting at the MRU end or aligning to cache lines don't gain much since the current eviction-slot reuse is already efficient, and Vec alignment is already handled by Rust's type alignment rules.

A more promising idea is splitting the links array into separate structures for probationary and protected items — this way traversing protected links stays within a smaller, warmer cache region. But that adds complexity since we'd need to track which array each index belongs to.

A cleaner alternative is adopting a slab-style free list where freed slots are kept sorted by index, and new allocations always take the lowest-index free slot. This clusters live items toward the beginning of the array, packing them more densely on cache lines. The tradeoff is the overhead of maintaining a sorted free list during allocation and eviction. The free-list approach is really just a latency win, and on workload-a where per-op-time is already zero, there's no real payoff. Let me explore a different angle that could actually move the hit-rate needle — maybe tweaking how probationary items are ordered during eviction, or looking at something like ARC's ghost list concept.

A ghost set would track recently evicted keys without their values, so if we see a put() that matches an evicted key, we can skip probationary and promote it straight to protected — giving it a second chance based on how recently it was evicted rather than hits within a window. The downside is that maintaining this ghost HashMap adds overhead to every operation.

I'm also wondering if we should just test different probationary capacity thresholds more systematically — we know capacity/8 beats capacity/4, so maybe capacity/16 would be worth trying. So a frequency sketch could actually work here because the sketch counts all accesses across the entire workload, not just hits within a probationary window. Even though workload-a's items don't get enough repeat hits while in the queue, they do get accessed multiple times overall, which means a sketch would capture that popularity signal. The key difference is that every `put()` call increments the sketch counter, giving us a more complete picture of item frequency. The real question is whether this helps on workload-a, where the high hit rate items are already protected and the problem is single-access items that never graduate. A frequency sketch wouldn't help those, but it would benefit multi-access items that get evicted from probationary before accumulating enough hits—they'd re-enter with a high count and graduate immediately. This feels like the right approach for H02: adding a small inline frequency sketch that counts across cache lifetimes, graduating items from probationary on their first hit if the sketch count is already ≥ 2. So if the frequency count is below 2, I should skip the promotion work entirely rather than refreshing the probationary position, since that would just delay eviction of a recently-inserted item. For protected items, I still relink on hits to maintain LRU order, but for probationary items below the threshold, I can avoid the unlink and push_back operations to save work on the common case of first accesses.

The implementation adds a frequency sketch to the cache struct, increments it on both new insertions and promotion attempts, then checks if the estimated frequency is at least 2 before graduating an item from probationary to protected. Items that don't meet the threshold stay in probationary without relinking, preserving FIFO order there.

The challenge is accessing the key in `promote()` since it only receives an index — we'd need to dereference `self.nodes[idx].key`. For the hash function with multiple seeds, we can't easily access the HashMap's hasher, so we'd need to use a fixed hasher instead.

I'm thinking of implementing a CountMin sketch with 4 seeds stored inline in the LruCache struct — using a 256-byte array to track frequency estimates. When a new key is inserted, we increment the sketch based on its hash; then in `promote()` for probationary items, we query the sketch and only graduate if the estimated frequency is at least 2. This adds minimal overhead to the struct size.

Looking at the fitness vector, the metadata-size score is based on the Node size (16 bytes), not the LruCache size, so growing the cache struct by 256 bytes doesn't directly impact that particular scoring dimension.

However, I'm concerned about per-operation performance. A 256-byte frequency sketch inline could push hot fields of LruCache across multiple cache lines, potentially hurting cache locality. I'm thinking of using a smaller sketch instead — 4 rows × 16 columns = 64 bytes total, which fits neatly in a single cache line. With 4 hash functions using modulo 16, I need to ensure good hash distribution for u64 keys.

For generic K: Hash + Eq types, I'll use a standard hasher to get a u64, then derive 4 independent hashes through prime-multiplied and shifted variations to index the 4 rows of 16 slots each.

Now I'm moving on to finalize the three hypotheses, starting with H01 — the extended quota probe, which currently protects 93.75% of the cache. I'm considering changing the `prob_cap` calculation from `(capacity / 8).max(1)` to `(capacity / 16).max(1)` to push the protection further.

H02 is a frequency sketch admission filter that adds a CountMin sketch to track access frequency across the entire trace lifetime, graduating probationary items only when their estimated frequency reaches 2 or higher, rather than waiting for two sequential hits within the probationary window.

For H03, I need to decide whether to cite a cross-cycle winner or explore from baseline. The instructions require at least one hypothesis to cite a winner, and since H01 and H02 already do that, H03 can cite baseline or a prior loser.

I'm considering SIEVE as a third strategy — it's a recently proposed eviction algorithm that uses a visited bit and hand pointer similar to CLOCK but simpler. When an item is accessed, the visited bit gets set; the hand scans forward, clearing visited bits when it finds them and evicting items with visited=0. This avoids the relinking overhead of LRU while achieving comparable hit rates.

The main challenge is fitting the visited bit into the Node struct. Since the current split design only has key and value fields, I'd need to either add a new field or encode the bit into one of the link pointers — perhaps using a high bit in the next pointer since valid indices are bounded. allow bit-stuffing easily. So for a baseline-derived SIEVE, I'd either accept the node size increase by adding a `visited` bool field, or use a separate bit vector to track visited status. The size tradeoff is real—adding the bool would push a Node<u64, u64> from 48 bytes to 56 bytes due to alignment padding.

Given that overhead, it's probably not worth implementing SIEVE from scratch on the baseline when the link-array split already proved to be a solid improvement. Let me explore a different approach for H03 instead—maybe LRU-K built on top of the v001 architecture.

I'm reconsidering what other policies haven't been tested yet. The cycle/0005/v002 winner uses the link-array split with a 75% protected quota and achieves strong hit-rates. But with cycle/0006/v001 now established as the baseline with 87.5% quota, I should make H03 meaningfully different from H01 and H02.

There are still open questions to explore—like whether pushing the quota even higher helps, or what other policy variations could differentiate the approaches.

For H03, I'm thinking about the probationary queue behavior. Currently, when a probationary item gets accessed, it immediately graduates to protected. But what if we allowed items to refresh their position within probationary without graduating—giving them a second chance to prove their value before committing them to the protected tier?

This is essentially what the SEEN_BIT approach was exploring. Then there's S3-FIFO, which uses a different architecture: a small queue for new items, a main queue for frequently accessed ones, and a ghost queue tracking recently evicted items. Items move between queues based on access patterns, with a visited bit controlling reinsertion behavior.

Looking back at the leaderboard, cycle/0003/v002 achieved the same 0.729 hit rate as the current best, but with different tradeoffs—metadata-size of 24 and per-op-time of 100, which didn't make it a winner overall. So there's precedent for hitting that hit-rate target through probationary capacity adjustments.

The current winner cycle/0006/v001 matches that 0.729 hit rate but with better structural efficiency: metadata-size of 16 and per-op-time of 0. We're at the same hit-rate ceiling, just more efficiently packed.

To break through that ceiling, I need a fundamentally better eviction strategy. Let me revisit cycle/0002/v001 as a parent—it used u32 sentinel links with 2Q and a 75% quota, keeping the Node structure compact at 24 bytes. I'm wondering if a different memory layout for the hot fields could unlock better performance.

Actually, let me step back and think about this more carefully. What if I implement SIEVE properly from scratch, using a dedicated bitfield for tracking visited state rather than embedding it in the node structure itself?

The core idea is straightforward: maintain a parallel `visited` array alongside the nodes, keep a hand pointer for the eviction scan position, and on cache hits just flip the visited bit without any relinking. When inserting a new key, I run the SIEVE eviction scan backwards from the hand position—if I encounter a visited slot, I clear it and keep scanning; if I find an unvisited slot, that's my eviction candidate. The list structure itself just tracks insertion order for the scan to traverse. I could use a bit-vector to pack the visited flags more efficiently—256 bytes instead of 2048 for 2048 elements. When a slot gets reused for a new item, I need to reset its visited flag to false. The doubly-linked list structure mirrors the baseline LRU, but since `get()` only sets a bit rather than relinking, it's much cheaper, though eviction requires scanning through potentially multiple nodes.

I'm exploring whether to store the visited bit directly in the link array—bit 31 of `next` is available since only `prev` uses the protection bit. But that creates a conflict with the NULL sentinel value, requiring careful masking when reading. A separate `visited` vector might be simpler. SIEVE should match or exceed LRU performance on Zipfian workloads based on the paper's results. This is a strong candidate for H03 since it's fundamentally different from the previous approaches—replacing the entire 2Q policy rather than tuning it. I'd keep the existing arena-based structure and HashMap for fast lookups, but swap in SIEVE's visited tracking mechanism instead of the 2Q probationary/protected queue logic.

The key changes involve adding a visited flag array parallel to the nodes, a hand pointer that scans backward through the list during eviction, and modifying the get operation to just mark items as visited rather than moving them around. New insertions go at the head so the scanning hand reaches them last, giving them a chance to be accessed before eviction. Defining the cache structure... Actually, I need to clarify the queue orientation. The SIEVE paper says items insert at the head and the hand starts at the tail, scanning toward the head—so head is newest and tail is oldest. The hand moves from old to new items, wrapping around when it reaches the head. For our implementation, I should align the doubly-linked list so head is the eviction end and tail is the MRU end, which matches what we already have. Implementing the core logic... `<usize>` is 8 bytes with niche optimization, so the baseline Node is 32 bytes total. Adding a `visited: bool` field bumps it to 40 bytes after padding, which hurts the metadata score compared to baseline. A better approach would be to reuse the link-array split from v001 and encode the visited bit into the `next` field itself. So a node with the visited flag set and next pointing to NULL becomes 0xFFFFFFFF, and one pointing to index 5 becomes 0x80000005. I can extract the next index with a bitwise AND against 0x7FFFFFFF and the visited flag by shifting right 31 bits. This keeps the Node struct at 16 bytes without any changes. I'll need to update the `unlink()` function to mask out the visited bit when reading the next pointer, and make sure to preserve that bit whenever I update the links.

Now I'm shifting the SIEVE implementation itself—removing the dual-queue structure since SIEVE operates on a single queue, storing the visited flag in bit 31 of next, and adding a hand field to track the eviction pointer. During gets, instead of promoting nodes, I just set the visited bit. For eviction, I'll scan through the queue SIEVE-style rather than popping from the front. The tradeoff is that dropping the 2Q structure might hurt hit rates compared to the 87.5% quota approach on Zipfian workloads, though SIEVE should still be competitive as a single-queue policy.

The real appeal would be faster per-operation time on hits—just setting a bit instead of relinking—but since per-op-time is already at the measurement floor of 0, there's no room to gain there. Combined with the risk of lower hit rates than 2Q, this feels like a risky bet. Let me look at other alternatives instead.

I'm considering a few other directions: using a compact frequency counter with a small hash table, testing the quota improvement on the older u32-sentinel baseline (though that's already been explored), or protecting recently-inserted items from probationary eviction for their first N seconds.

The LIRS-inspired approach would give new items a grace period where they can't be evicted—I'd track insertion order with a counter and skip evicting items inserted within the last K insertions. The downside is storing an insertion timestamp per node adds 8 bytes to each node, growing the metadata overhead from 16 to 24 bytes, which is a meaningful cost.

Even using a u32 instead of u64 doesn't help since the node is already tightly packed at 16 bytes. This runs into the same fundamental constraint as the SEEN_BIT approach—any per-node state requires growing the node structure.

Let me explore a different angle that avoids adding per-node metadata entirely. There's an alternative from earlier work: instead of immediately demoting protected items when the quota overflows, I could check if probationary is under 1% capacity first before triggering the demotion, which would reduce the eviction cost.

But that won't improve the per-operation time since it's already at zero, and it would likely hurt hit-rate by keeping more items in protected during the overflow window before batch demotion kicks in.

Another option is to use random sampling in probationary instead of strict FIFO eviction—pick k items at random and evict the one with the least recency signal. Though without per-item recency tracking in probationary, random sampling ends up being equivalent to just random eviction anyway.

I could also try making probationary an LRU queue instead of FIFO, where accessed items refresh their position within probationary rather than graduating immediately. But that's essentially the same as the SEEN_BIT two-hit filter approach, which has already been ruled out.

Let me think about what actually hasn't been explored yet across all the cycles. The issue is that with random slot reuse, live items get scattered across the arrays, causing cache misses during traversal. I'm exploring a free list approach that preferentially allocates low-index slots to cluster live items near the beginning, improving cache locality. The implementation uses a free stack that collects evicted slot indices and reuses them in order during insertion.

However, truly clustering items would require a priority queue to always pop the lowest-index slot, which adds O(log n) overhead per operation—likely not worth it given the measurement floor makes any benefit invisible. A simpler LIFO stack approach is essentially equivalent to the current immediate-reuse behavior anyway.

Given these constraints, I'm stepping back to reconsider what H03 should actually target, looking at the incumbent best hit-rate of 0.7291.

The global best ever achieved was 0.729, and nothing has surpassed it yet. To break through that ceiling, I need a more selective eviction policy—specifically, I need to understand which items are being evicted that shouldn't be. In the current 2Q setup with 87.5% protected quota, new items enter a small probationary region and get evicted if they don't get a hit before being displaced, while items that do get a hit graduate to the larger protected region. The frequency sketch in H02 attempts to reduce false evictions from probationary, but maybe the real lever is extending the probationary window itself or adjusting how items transition between regions.

I'm considering a warm-up phase approach where during the initial fill of the cache (before it reaches capacity), all new items skip probationary and go straight into protected, treating every insertion as an immediate graduation. Once the cache fills up, the system switches back to normal 2Q behavior. The challenge is that the trace starts fresh each time, so I need to think through whether this warm-up strategy actually helps or just delays the real problem.

The warm-up period is likely negligible compared to the millions of accesses in the trace, so the steady-state behavior is what really matters for hit-rate. I'm also reconsidering whether dynamically adjusting the probationary queue size based on the ratio of probationary hits to total hits would help, but on stable workloads like workload-a with Zipfian distribution, this adaptive approach probably won't provide any benefit.

Now I'm thinking about trying SIEVE with the baseline structure instead. Even though the Node size would grow to 40 bytes with the visited flag, SIEVE should match LRU's hit-rate around 0.665 on Zipfian and improve per-operation time since there's no relinking on cache hits. That said, it still won't reach 2Q's 0.729 hit-rate. To beat that, I need a fundamentally better policy. LFU is theoretically near-optimal for Zipfian since frequency correlates with future access probability, but it comes with O(log n) eviction costs and requires frequency counters—though that could be implemented with a HashMap.

TinyLFU, which Caffeine uses internally with a CountMin sketch for frequency estimation plus window and protected queues, is another option. LRU-K variants like LRU-2 track the K-th most recent access time per item and perform better than standard LRU on Zipfian, but storing two timestamps per item would grow the node size further. Given that 2Q is already quite close to optimal for Zipfian workloads, the marginal gains from more sophisticated frequency-based policies might be minimal.

Looking at the memory layout, the current implementation allocates the nodes and links vectors upfront—for a capacity of 2048, that's 32KB for nodes and 16KB for links. The nodes array spans many cache lines, and traversing the LRU chain jumps randomly through both arrays, which hurts cache locality. One approach would be to cluster frequently accessed items toward the beginning of the arrays to improve L1/L2 cache hit rates.

Periodic compaction during eviction could help—moving live items to fill gaps left by evictions—but that requires updating the map and becomes O(N) in the worst case, so it's too complex.

Instead, I'm thinking about testing different probationary queue sizes to find where the hit-rate reversal point actually is. We've seen that capacity/4 gives 0.720 hit-rate and capacity/8 gives 0.729, so I want to explore what happens at capacity/2 (50% protected) and capacity/16 (93.75% protected) to establish the full curve.

Wait—I just realized cycle/0002/v001 already tested capacity/4 and got 0.720, so this isn't actually a new experiment; it's already been run.

Looking at the data more carefully, the real comparison is between cycle/0005/v002 (link-array split with 75% quota yielding 0.720) and cycle/0006/v001 (same structure but 87.5% quota yielding 0.729). The quota increase alone accounts for the improvement, not the structural change.

So I'm settling on three concrete hypotheses: pushing the quota further to 93.75% to see if the gains continue, implementing a frequency sketch for admission decisions to overcome the SEEN_BIT window limitation, and investigating something entirely different from the non-winning cycle/0003/v002 variant which also achieved 0.729.

That variant used a larger node size but got beaten on metadata and performance. I'm wondering if I can combine its quota enlargement strategy with the link-array structure from the current winner—but wait, that's already what cycle/0006/v001 is. So I need to dig into what exactly cycle/0003/v002 changed with its probationary cap to see if there's a different angle worth exploring.

Actually, I should stop chasing that thread. Let me consider some concrete alternatives: a flat open-addressing cache without the linked list overhead, though CLOCK-style eviction has already failed twice. Or maybe an OPT-style optimal ordering, but that's not feasible for an online policy. Another option is to change how new items are inserted into the probationary section—instead of treating them as FIFO, what if I mark them as recently-used when they first arrive?

The real issue is distinguishing between new items and returning items that were previously evicted. Without a ghost set, I can't track that distinction. But here's an idea: what if I make the probationary section itself an LRU queue where accessing an item refreshes its position without graduating it to protected? An item would only graduate after a second hit while in probationary—first hit refreshes its position, second hit promotes it.

Wait, that's just the SEEN_BIT mechanism implemented differently, and the cycle notes say SEEN_BIT is retired anyway.

Let me shift focus to the `put()` path for updating existing keys. Currently, updating a key's value triggers the same promotion logic as a `get()` would. What if I skip that promotion on updates? An item would stay in its current queue position when its value is updated, saving the cost of unlinking and reinserting. The question is whether this hurts the hit-rate—an item could remain in probationary even after being written to, making it more vulnerable to eviction.

In a write-heavy workload this could matter, but for read-heavy Zipfian traces it probably doesn't. The per-op-time is already at the measurement floor, and the hit-rate might dip slightly, so this doesn't feel like a strong enough win.

Instead, I should probe whether `prob_cap = capacity/6` (roughly 83% protected quota) lands closer to the true optimum. I have two data points: 75% protected gives 0.720 hit-rate, and 87.5% protected gives 0.729. Let me check if the peak is somewhere between those two. I'm realizing the gradient between 75% and 87.5% is monotonic—going to 83.3% won't beat the 0.729 we already have at 87.5%, so this hypothesis doesn't hold up. I should abandon this direction and look for something genuinely different from the baseline instead.

Looking at what we know about the workload: it's Zipfian with heavy-hitters, many single-access items that break SEEN_BIT, and a stable distribution that defeats adaptive quotas. The 2Q policy at 87.5% quota gets us to 72.9%, but I'm wondering if there's a better approach. LIRS could theoretically do better on Zipfian by tracking inter-reference recency, though it's complex to implement with its large history stack. Another angle is LFU approximations like TinyLFU or W-TinyLFU, which maintains a small window to balance recency and frequency.

I'm also considering a simpler approach from the earlier work: using a frequency counter embedded in the existing node structure. The `prev` field currently uses bit 31 for the protection bit and bits 0-30 for the previous index, so I could potentially use bits 29-28 to track frequency information without adding extra memory overhead.

The tradeoff is that this would reduce the maximum addressable index range, but that's not a practical constraint. The real complexity comes from handling NULL pointers and ensuring the bit encoding doesn't conflict with existing sentinel values. Looking at the current winning implementation, the Link struct keeps things straightforward with separate `prev` and `next` fields, which suggests the simpler approach might be overcomplicating things. Redefining NULL to 0x3FFFFFFF lets me use bits 31-30 as a 2-bit saturating counter for frequency tracking without increasing struct size, keeping the index bits to 0-29. The graduation threshold would then require a counter value of at least 1, meaning the node needs two accesses total before promotion.

The complication is that `promote()` calls `unlink()` and `push_back_prot()`, which both read and write the `next` pointer—I'd need to preserve the counter bits through those operations rather than overwriting them.

Let me reconsider the approach entirely and focus on a third hypothesis that's distinct from the quota probe and frequency sketch ideas, something concrete and testable that doesn't rely on retired mechanisms like SEEN_BIT or adaptive clocking.

H03 is about experimenting with a multi-queue demotion policy: instead of moving demoted items to the MRU end of probationary (where they'd be last to evict), I'd insert them at the LRU end so they face earlier eviction. Items that failed to maintain protected status shouldn't get maximum residence time in probationary—they should be vulnerable sooner.

But checking the 2Q paper and the actual code, I see `push_back_prob()` is putting demoted items at the MRU end, which contradicts what the paper describes. Let me trace through the implementation to understand what's actually happening.

The key insight is that demoting to the MRU end protects demoted items longer, but what if we flipped that and demoted to the LRU end instead? The reasoning would be that items already failed to get re-accessed in protected, so they should be evicted quickly to free up space for genuinely new items that might have better hit potential. Writing the implementation...

Now I'm questioning whether demoting items to the front of the probationary list actually helps. The current approach keeps demoted items at the MRU end where they survive longer and get another chance to graduate if accessed again. Moving them to the LRU end would evict them faster, which only helps if they're genuinely unpopular—but if they're items that were popular enough to graduate initially, evicting them prematurely might hurt hit-rate on workload-a.

The real tension is about probationary slot allocation: with only 256 slots available under the 87.5% protected quota, demoted items occupying the MRU end means fewer slots for new insertions. Demoting to the LRU end would free up space for new items to enter at the MRU end and survive longer, giving them a better shot at getting hits and graduating. This is the case for trying the LRU approach, though I need to check what the original 2Q paper actually recommends. to the LRU end of probationary instead of MRU end, which should reduce queue clogging from failed-protected items. I'm settling on three final hypotheses to test: pushing the protected quota higher to 93.75%, adding a frequency sketch to graduate items only when they've been accessed multiple times, and adjusting where demoted items land in the probationary queue. The 87.5% quota is already proven better, so I should test the demote-to-LRU-end improvement on that best baseline. This means all three hypotheses—H01, H02, and H03—will branch from cycle/0006/v001, each exploring orthogonal modifications: quota tuning, frequency-sketch admission filtering, and demotion position changes respectively.

Now I'm drafting the hypothesis files to document each experiment clearly.

**H01.md:** The change tightens the probationary pool from 12.5% down to 6.25% of capacity by adjusting the divisor, which automatically cascades through the protected-quota calculation. All the core functions stay the same. The hypothesis predicts that hit-rate improvements continue scaling monotonically as we shrink the probationary zone, provided we don't drop below a viable minimum.

**H02.md:** Adding a frequency sketch to track global item popularity and filtering out low-frequency items from probationary graduation... Implementing the frequency sketch helper... Refining the `promote()` method to avoid borrow conflicts... The simplest fix is to clone the key before calling `sketch_inc`, since `K: Clone` is already required. For `put()` with new keys, I can call `sketch_inc` before moving the key into the map. I'm also reconsidering whether to refresh a probationary item's position on cache hits—doing so adds overhead, and I need to think through whether the frequency justifies the extra work.

Looking at the frequency counting: when we increment in both `put()` and `promote()`, a new key starts at frequency 1, then hits frequency 2 on its first cache access, which immediately graduates it if the threshold is 2. That means there's no filtering effect at all—every probationary item graduates on first hit. I'd need a threshold of at least 3 to filter anything, or I should skip counting the initial `put()` increment.

The key realization is that the frequency sketch persists across evictions. If an item gets evicted and reinserted, its frequency count from before is still there. So if I don't increment during re-insertion, a previously-seen item would only need one more hit to graduate. But if I do increment on re-insertion, it counts as a fresh start, and the first probationary hit still graduates it immediately. back. On Zipfian traces, popular items cycle in and out of the cache, so returning items that were previously evicted could benefit from the sketch's memory of their prior access history. On workload-a with a 72.9% hit rate, the 27.1% misses split between new items (which the sketch doesn't help) and returning items (which it does). If returning items make up a significant portion of misses—which is plausible on Zipfian traces where moderate-frequency items constantly cycle—the sketch could provide real benefit. Now I'm refining H02's threshold logic to clarify when the frequency counter increments during insertion versus promotion.

With threshold=2, incrementing on both initial insertion and probationary hits means first-time items graduate on their first probationary hit (freq: 1→2), and returning items also graduate immediately (freq: ≥2→≥3). This gives no filtering effect at all—every probationary hit graduates, same as the current behavior. I'd need threshold=3 instead to actually filter: first-time items stay in probationary after their first hit (freq: 1→2, below 3), while returning items with prior history graduate (freq: ≥2→≥3).

This creates the asymmetry I want—new items need two probationary hits to graduate while returning items get a fast track—but it's the same two-hit requirement that failed with the SEEN_BIT approach. The real question is whether enough returning items benefit from this fast track to make up for the cost of keeping new items in probationary longer.

Actually, I'm reconsidering: instead of incrementing in `put()`, I should only increment on `get()` hits. That way first-time items need two hits to graduate, but returning items that have accumulated frequency across multiple cache visits will already have count ≥ 2 from prior visits, giving them the fast track on their next insertion.

The sketch stays permanent across the entire cache history, which is fundamentally different from SEEN_BIT's single-window requirement. I'll go with this for H02, though I need to think through whether protected hits should also increment the sketch since they signal popularity.

For now, I'll keep it simple: only increment on probationary hits during the promotion check, since protected items don't need sketch updates anyway. Items that get demoted back to probationary retain their accumulated counts. But I'm reconsidering whether new insertions should also increment the sketch to track first-time arrivals properly—if I don't, a fresh item starts at count=0 and needs two probationary hits to graduate, which seems right. across windows. If items cycle in and out of the cache, their counts accumulate in the sketch over time, so moderate-popularity items will eventually have count≥1 from prior visits—fast-tracking them to protected status on their next hit. The real question is whether this cross-window persistence actually moves the needle on workload-a, given that the cycle analysis already showed insufficient within-window repeats.

I'm settling on H02 with a frequency sketch that persists across evictions, increments only on successful gets in the probationary zone, and graduates items once their count reaches 2. Skipping increments on protected-zone gets to keep it simple.

Actually, I'm reconsidering — if I only increment for probationary hits, then protected items that eventually get demoted back to probationary will have stale sketch counts from their earlier probationary phase, making them harder to fast-track on re-entry. It's better to increment on both probationary and protected hits, even though it adds a small cost per protected access.

But looking at the workload data, most hot items are already in protected with a 72.9% hit rate, so adding sketch_inc overhead to all those protected hits could hurt per-op-time significantly — potentially as much as the SEEN_BIT approach did. To keep overhead minimal, I'll only increment during the probationary promote path, right before the graduation decision. Once an item reaches protected, I don't need its frequency estimate anymore, so skipping the increment there saves cost without sacrificing the core benefit.

The tradeoff is that first-time items still need two probationary hits to graduate (same as SEEN_BIT), but returning items with prior history only need one hit — that memory advantage is what makes the frequency sketch potentially better than SEEN_BIT. Now I need to implement this and figure out the hashing strategy for the sketch key.

Writing implementation code... Writing hash derivation...

To handle the borrow checker issue, I'm cloning the key before using it in methods that need mutable self references—for Copy types like u64 this is essentially free, but for other types it requires a clone operation.

Now I'm ready to write out the hypothesis files. For H03, I'm changing the demotion strategy: instead of pushing demoted protected items to the MRU end, I'll push them to the LRU end when quota overflow occurs, and I need to implement a new function to handle this front insertion with probability. Finishing the function and reconsidering the PROT_BIT logic—since items in the probationary list always have PROT_BIT=0 in their prev field, setting the current head's prev to idx32 (which has no high bit set) correctly maintains that invariant. The PROT_BIT is tied to the node itself, not its predecessor, so when a new node enters the probationary queue, its `prev` is NULL with PROT_BIT = 0, and the current head's `prev` gets updated to point to the new node while preserving its own PROT_BIT. Now I'm ready to write out the implementation files. Continuing with the probationary hit logic: I'm checking the frequency sketch to decide whether to promote the entry to protected status. If the frequency is below the threshold, I leave it in the probationary queue untouched to preserve FIFO ordering. Otherwise, I unlink it from probationary, move it to the protected queue, and then enforce the protected capacity budget by evicting from the head if needed.

Now I'm implementing the sketch increment function that hashes the key and derives multiple hash positions using bit mixing to update frequency counters. Finishing the frequency sketch update logic...

I'm verifying that `DefaultHasher::new()` works correctly and considering how the randomized seeding affects the sketch. Within a single program run, the hash positions remain consistent, which is what matters for the cache's lifetime. The randomization across runs doesn't impact functionality here.

Now I'm ready to write out the files and checking that the output directory path exists.

### 18:44:05 chat

Now I have all the context I need. Let me write the three hypothesis files.

### 18:44:06 tool: Bash

```json
{
  "command": "ls \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0007\\hypotheses\" 2>/dev/null || echo \"DIR_MISSING\"",
  "description": "Check if output directory exists"
}
```

### 18:44:19 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0007\\hypotheses\\H01.md",
  "content": "# H01\n\nReducing prob_cap from capacity/8 to capacity/16 (pushing the protected quota from 87.5% to 93.75%) will further improve hit-rate on the Zipfian workload by reducing demotion pressure for moderately-frequent items, continuing the monotonic gain seen from the 75% → 87.5% transition.\n\n## Parent\n\ncycle/0006/v001\n\n## Changes\n\nIn `try_new()`, change the single line that computes `prob_cap`:\n\n```rust\n// Before:\nlet prob_cap = (capacity / 8).max(1);\n\n// After:\nlet prob_cap = (capacity / 16).max(1);\n```\n\nAt capacity=2048 this shrinks the probationary pool from 256 slots (12.5%) to 128 slots (6.25%), expanding the protected quota to 93.75%. All downstream logic is unaffected: `promote()` derives `prot_cap = self.capacity - self.prob_cap` directly, so the new quota propagates automatically. `evict_one()` still drains probationary first and falls back to protected — the fallback path becomes more reachable but the code path is unchanged.\n\nUpdate the inline comment at the top of the file (policy sketch section) and the field-declaration doc comment for `prob_cap` to reflect the new divisor and resulting percentages.\n\nNo other changes. `Node<K,V>`, `Link`, `LruCache` struct layout, and all methods remain identical to cycle/0006/v001.\n"
}
```

### 18:44:44 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0007\\hypotheses\\H02.md",
  "content": "# H02\n\nAdding a persistent 64-byte CountMin frequency sketch to LruCache and using it to gate probationary graduation — requiring an estimated global frequency of at least 2 before promoting to protected — will improve hit-rate over cycle/0006/v001 by fast-tracking returning items (which have prior sketch history) without the within-window constraint that caused SEEN_BIT to fail.\n\n## Parent\n\ncycle/0006/v001\n\n## Changes\n\n### Struct change\n\nAdd one field to `LruCache<K, V>` (after `capacity`):\n\n```rust\nfreq_sketch: [u8; 64],  // 4-row × 16-col CountMin sketch; zero-initialized\n```\n\n64 bytes = one cache line. `Node<K,V>` and `Link` are unchanged (still 16 bytes and 8 bytes respectively).\n\nIn `try_new()`, initialise the new field:\n\n```rust\nfreq_sketch: [0u8; 64],\n```\n\n### New private method: `sketch_inc`\n\nAdd to the `impl<K,V>` block:\n\n```rust\nfn sketch_inc(&mut self, key: &K) -> u8 {\n    use std::hash::{Hash, Hasher};\n    use std::collections::hash_map::DefaultHasher;\n    let mut h = DefaultHasher::new();\n    key.hash(&mut h);\n    let v = h.finish();\n    // Derive four independent column indices (0..16) from one hash\n    // by multiplying with four different primes and taking low 4 bits.\n    let cols: [usize; 4] = [\n        ( v                                   & 0xF) as usize,\n        ((v.wrapping_mul(0xbf58476d1ce4e5b9)) & 0xF) as usize,\n        ((v.wrapping_mul(0x94d049bb133111eb)) & 0xF) as usize,\n        ((v.wrapping_mul(0x9e3779b97f4a7c15)) & 0xF) as usize,\n    ];\n    let mut min_val = u8::MAX;\n    for (row, &col) in cols.iter().enumerate() {\n        let slot = &mut self.freq_sketch[row * 16 + col];\n        *slot = slot.saturating_add(1);\n        if *slot < min_val { min_val = *slot; }\n    }\n    min_val\n}\n```\n\nThis method increments all four row-cells and returns the post-increment minimum (the CountMin frequency estimate after this access).\n\n### Change to `promote()`\n\nIn the probationary branch of `promote()` (the `else` arm, currently starting with `self.unlink(idx); self.prob_len -= 1; ...`), insert a frequency check BEFORE unlinking:\n\n```rust\n} else {\n    // Probationary hit: consult the frequency sketch.\n    // Clone key to satisfy the borrow checker (Copy for u64, cheap for all K).\n    let freq = {\n        let k = self.nodes[idx].key.clone();\n        self.sketch_inc(&k)\n    };\n    if freq < 2 {\n        // Below threshold: item has only been accessed once globally.\n        // Leave it in the probationary FIFO (no relinking needed) and return.\n        return;\n    }\n    // freq >= 2: item has recurring access history — graduate normally.\n    self.unlink(idx);\n    self.prob_len -= 1;\n    self.push_back_prot(idx);\n    self.prot_len += 1;\n    let prot_cap = self.capacity - self.prob_cap;\n    if self.prot_len > prot_cap {\n        let h = (self.prot_head & IDX_MASK) as usize;\n        self.unlink(h);\n        self.prot_len -= 1;\n        self.push_back_prob(h);\n        self.prob_len += 1;\n    }\n}\n```\n\nThe protected hit branch (`if protected { ... }`) is unchanged — no sketch access for protected hits.\n\n### Why this differs from SEEN_BIT\n\nSEEN_BIT required a second hit within one contiguous probationary window; the bit was cleared on eviction. The sketch is permanent: an item evicted with freq=1 retains that count, so on its next probationary visit its first hit pushes freq to 2 and it graduates immediately. Only true one-time items (never seen again after eviction) are held back by the two-hit threshold. Workload-a's moderate-frequency Zipfian items cycle in and out of the cache; those should accumulate freq ≥ 2 quickly across visits.\n\n### Invariants to preserve\n\n- `self.prob_len + self.prot_len == self.map.len()` — maintained because the early-return path does not change queue membership.\n- `self.prob_len > 0` when evicting from probationary — unaffected; the sketch only changes the graduation rate, not the eviction path.\n- No changes to `evict_one()`, `unlink()`, `push_back_prob()`, `push_back_prot()`, or `put()`.\n"
}
```

### 18:45:06 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0007\\hypotheses\\H03.md",
  "content": "# H03\n\nDemoting overflow-protected items to the LRU end (oldest position) of the probationary queue — instead of the current MRU end — will improve hit-rate by evicting failed-protected items quickly and preserving probationary slots for fresh insertions that have not yet had any access opportunity.\n\n## Parent\n\ncycle/0006/v001\n\n## Changes\n\n### New method: `push_front_prob`\n\nAdd alongside the existing `push_back_prob` in the `impl<K,V>` block:\n\n```rust\n/// Prepend a node to the LRU end (eviction-candidate end) of the probationary queue.\n/// Used when demoting an overflow-protected item so it is evicted before newer probationary entries.\n/// Clears PROT_BIT in `prev` (PROT_BIT = 0 → probationary).\nfn push_front_prob(&mut self, idx: usize) {\n    let idx32 = idx as u32;\n    // This node has no predecessor: prev = NULL with PROT_BIT = 0.\n    self.links[idx].prev = NULL;\n    self.links[idx].next = self.prob_head;\n    if self.prob_head != NULL {\n        // Update the old head's prev: preserve its own PROT_BIT (should be 0 for\n        // probationary nodes, but mask-and-or to be safe) and set index = idx32.\n        let old_head = self.prob_head as usize;\n        let old_prot = self.links[old_head].prev & PROT_BIT;\n        self.links[old_head].prev = old_prot | idx32;\n    } else {\n        // Probationary queue was empty: this node is also the tail.\n        self.prob_tail = idx32;\n    }\n    self.prob_head = idx32;\n}\n```\n\n### Change to `promote()`\n\nIn `promote()`, inside the probationary graduation branch, the demotion of an overflow-protected item currently calls `push_back_prob` (MRU end). Replace it with `push_front_prob` (LRU end):\n\n```rust\n// Before:\nself.push_back_prob(h);\n\n// After:\nself.push_front_prob(h);\n```\n\nThe surrounding context (inside the `if self.prot_len > prot_cap` block) remains identical:\n\n```rust\nif self.prot_len > prot_cap {\n    let h = (self.prot_head & IDX_MASK) as usize;\n    self.unlink(h);\n    self.prot_len -= 1;\n    self.push_front_prob(h);   // ← changed from push_back_prob\n    self.prob_len += 1;\n}\n```\n\nNo other code changes. All other callers of `push_back_prob` — in `put()` (new insertion) and `evict_one()` fallback — remain unchanged.\n\n### Rationale for correctness\n\n`push_front_prob` sets the new node's `prev` to `NULL` with `PROT_BIT = 0`, correctly marking it as the probationary head with no predecessor. The existing head's `prev` index is updated to `idx32` (also PROT_BIT = 0, which is correct for a probationary node). `unlink()` reads `PROT_BIT` from `prev` to dispatch to the correct head/tail pointer; since the demoted node's `prev = NULL` (no PROT_BIT), `unlink()` will correctly update `prob_head` when this node is eventually evicted.\n\n### Expected effect\n\nCurrently, demoted items occupy the MRU end of probationary and survive until all newer probationary items are evicted — often outlasting many fresh insertions that never get a second access. By placing demoted items at the LRU end, they are evicted on the next probationary eviction call, freeing the slot for a new insertion. New insertions (at the MRU end) then survive longer, increasing the window in which they can receive a hit and graduate. On a Zipfian workload where the moderate-frequency population is large relative to the 12.5% probationary budget, this reordering may meaningfully reduce wasted probationary residence by expired-protected items.\n"
}
```

### 18:45:18 chat

Three hypothesis files written to the output directory. Here is the cohort summary:

---

**H01 — Quota push to 93.75%** (`cycle/0006/v001`)  
Single-line change: `prob_cap = (capacity / 16).max(1)` instead of `/8`. Tests whether the hit-rate climb from 75% → 87.5% continues to 93.75%, or whether the probationary pool becomes too thin to serve its staging function.

**H02 — Persistent CountMin frequency sketch** (`cycle/0006/v001`)  
Adds a 64-byte `[u8; 64]` (4 rows × 16 columns) CountMin sketch to `LruCache`. The sketch is incremented only on probationary hits (in `promote()`), never on `put()` or protected hits. Graduation requires estimated frequency ≥ 2. The critical distinction from SEEN_BIT: the sketch is permanent across eviction cycles, so a moderate-frequency item that entered the cache previously with freq=1 will fast-track to protected on its next probationary hit (freq goes to 2 on first hit after re-entry), while a true one-time wonder stays blocked until a second hit.

**H03 — Demote-to-LRU-end** (`cycle/0006/v001`)  
Adds `push_front_prob()` (prepend to the LRU/oldest end) and replaces the single `push_back_prob(h)` call inside the protected-overflow demotion block with `push_front_prob(h)`. Demoted items are now evicted on the very next probationary eviction, freeing slots for fresh insertions that have not yet had any access opportunity. No struct changes; `Node<K,V>` stays 16 bytes.


---

- finished: 2026-07-19T18:45:18+00:00
- duration: 1267.17s
- status: ok
- input_tokens: 15
- output_tokens: 64721
- cache_read_tokens: 488206
- cache_write_tokens: 59410
- cost_usd: 1.341655
