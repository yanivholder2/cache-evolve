# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0010\hypotheses
- allow_edits: True
- started: 2026-07-21T08:55:43+00:00

## timeline

### 08:55:54 thinking

Let me read all the context files to understand the current state of the research and then generate 3 distinct hypotheses for cycle 0010.

### 08:56:00 tool: Agent

```json
{
  "description": "Read all context files for cycle 0010 hypothesis generation",
  "prompt": "Read the following files and return their complete contents. I need all of them to generate hypotheses for an evolve research loop.\n\nFiles to read:\n\n1. C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\cycle-winners.json\n2. C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\cycle-insight.md\n3. C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\leaderboard.md\n4. C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles-ledger.jsonl\n5. C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cross-cycle-insights.md (if present)\n6. C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline (list all files and read them all)\n\nAlso list all variant directories under:\nC:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\\n\nThen for each variant directory found (v001, v002, v003, etc.), read:\n- The overlay files in the latest attempt directory\n- The fitness-vector.json\n- The insight.md\n\nReturn ALL file contents verbatim so I can synthesize hypotheses."
}
```

### 08:56:02 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\cycle-winners.json"
}
```

### 08:56:04 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\cycle-insight.md"
}
```

### 08:56:04 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\leaderboard.md"
}
```

### 08:56:04 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles-ledger.jsonl"
}
```

### 08:56:06 tool: Bash

```json
{
  "command": "if (Test-Path \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cross-cycle-insights.md\") { Get-Content \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cross-cycle-insights.md\" } else { Write-Output \"FILE NOT PRESENT\" }"
}
```

### 08:56:06 tool: Bash

```json
{
  "command": "Get-ChildItem \"C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline\" -Recurse | Select-Object FullName"
}
```

### 08:56:07 tool: Bash

```json
{
  "command": "Get-ChildItem \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\\" -Directory | Select-Object Name"
}
```

### 08:56:11 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants"
}
```

### 08:56:12 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline"
}
```

### 08:56:13 tool: Glob

```json
{
  "pattern": "cross-cycle-insights.md",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995"
}
```

### 08:56:16 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cross-cycle-insights.md"
}
```

### 08:56:17 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline\\insight.md"
}
```

### 08:56:18 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline\\eval\\fitness-vector.json"
}
```

### 08:56:18 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline\\eval\\fitness\\hit-rate.json"
}
```

### 08:56:18 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline\\eval\\fitness\\metadata-size.json"
}
```

### 08:56:19 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline\\eval\\fitness\\per-op-time.json"
}
```

### 08:56:19 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\lru_cache\\baseline\\eval\\caffeine\\hit_rate_2048.csv"
}
```

### 08:56:20 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\v001\\insight.md"
}
```

### 08:56:21 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\v002\\insight.md"
}
```

### 08:56:22 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\v003\\insight.md"
}
```

### 08:56:22 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\v001\\eval\\fitness-vector.json"
}
```

### 08:56:24 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\v002\\eval\\fitness-vector.json"
}
```

### 08:56:24 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\v003\\eval\\fitness-vector.json"
}
```

### 08:56:25 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\v001\\attempts\\01\\overlay\\src\\lru_cache\\mod.rs"
}
```

### 08:56:26 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\v002\\attempts\\01\\overlay\\src\\lru_cache\\mod.rs"
}
```

### 08:56:26 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0009\\variants\\v003\\attempts\\01\\overlay\\src\\lru_cache\\mod.rs"
}
```

### 09:04:21 thinking

Now I have all the context I need to generate 3 distinct hypotheses for cycle 0010. Let me synthesize what I know:

## Key Findings from Cross-Cycle History

**Incumbents (winners to build from):**
- `cycle/0006/v001`: hit-rate=0.7291, metadata-size=16, per-op-time=0
- `cycle/0007/v002`: hit-rate=0.7291, metadata-size=16, per-op-time=0
- `cycle/0005/v002`: hit-rate=0.7202, metadata-size=16, per-op-time=0
- 16-byte Node via link-array split has proven solid across multiple cycles
- 87.5% protected quota (capacity/8 probationary) works as a cost-free ceiling
- Two-queue policy with spare-bit encoding is the foundation

**What hasn't worked:**
- CLOCK/sweep eviction and two-hit admission both failed
- Pushing quota above 87.5% consistently causes latency regression across independent tests
- Demotion batching in cycle 0009 didn't help
- FNV-1a hasher speedup exists but doesn't compensate for the quota increase overhead

**Still need to explore:**
- FNV-1a in isolation at the 87.5% quota baseline, since previous tests mixed it with other changes
- Using a frequency sketch to order evictions within the probationary queue based on estimated access patterns
- Cache-line alignment and arena packing optimizations for the 16-byte nodes
- A lazy-promotion segmented LRU approach as an alternative policy

The cycle 0009 results showed all variants exceeded the quota threshold, and FNV-1a was only tested alongside the quota increase. The recommendation is to isolate FNV-1a against the clean 87.5% baseline to see if the hasher alone improves performance without structural changes.

For cycle 0010, I should test FNV-1a at 87.5% quota first—replacing RandomState with a zero-sized BuildHasher struct should trim the ruCache by 16 bytes and potentially lower per-op time through cheaper hashing while preserving the 0.7291 hit-rate. Then explore using a frequency sketch for smarter eviction within the probationary queue, prioritizing items with lower estimated frequency to protect emerging hot items. Finally, investigate structural changes around node alignment or cache sizing. Now I'm considering a frequency-ordered probationary eviction strategy that uses a lightweight frequency counter to track access patterns within the probationary queue, so when it overflows I can evict the coldest items rather than just the least recently used ones—this should improve hit-rate by keeping warmer items around longer.

Let me verify what the parent cycles actually implemented by checking their leaderboard results, since both cycle/0006/v001 and cycle/0007/v002 show identical fitness scores despite being separate iterations. Since cycle 0007/v002 is the best incumbent at 0.7 hit-rate with a 16-byte Node and 87.5% protected quota, I can reference the specific modifications from cycle 0009's variants. Looking at v003, I see how FNV-1a hashing was integrated by adding custom hasher structs and switching the HashMap to use them with capacity initialization, which freed up 16 bytes from the LruCache structure itself.

For H02, I'm exploring a probationary eviction strategy with frequency bias using a small window scan—when the probationary queue overflows, instead of strictly evicting the tail, I'd pick between the tail and tail-1 based on which key hashes to a lower frequency bucket in a compact 2-bit counter array of 256 entries. This adds only about 32 bytes of overhead while avoiding the graduation-gate failure mode by working within the existing eviction flow.

For H03, I'm considering a lazy-demotion approach where protected queue overflow doesn't immediately demote items—instead, demotion gets deferred until the item is accessed again or the next eviction trigger fires, giving items a second chance to be re-accessed before actually moving them down to probationary.

But that's getting complex with extra branches. A cleaner angle might be cache-line alignment for the node pool—since 16-byte nodes fit four per 64-byte cache line, aligning the arena allocation could improve prefetch behavior as a pure latency win. The current implementation uses a HashMap with u32 indices pointing into a node vector, so I'd want to ensure that vector starts at a properly aligned boundary.

Actually, pre-populating the node pool to exactly the capacity on creation would eliminate Vec growth reallocations during the first insertions, smoothing out those occasional realloc spikes.

But the incumbents already measure per-op-time at the floor, so this wouldn't show measurable improvement and wouldn't help hit-rate either.

A better approach for H03 is a small circular buffer of recently-evicted keys—a ghost entry mechanism. When a key that was recently evicted gets inserted again, it skips the probationary queue and goes straight to protected admission. This targets hit-rate by avoiding the need to re-warm hot items that were evicted due to capacity pressure. It's distinct from the two-hit gate because it's specifically about items that were previously warm enough to reach protected status, not just items seen twice in the current window.

The overhead is minimal—just 512 bytes for 64 keys in a fixed-size ring buffer. But I'm wondering if this faces the same fundamental issue as the two-hit gate: the workload doesn't generate enough repeat accesses within a window. Ghost entries don't require within-window repeats though—they're about items that were hot enough to be in protected but got evicted. If the Zipfian hot set is stable, those items would cycle between being in and out of the cache.

However, if the hot set is truly stable and the cache is large enough (with 87.5% protected), items from the protected queue shouldn't be evicting frequently—only the least-hot items at the tail when it overflows. At 2048 capacity, the Zipfian hot set seems well-served already. This approach might be risky.

The frequency-sketch idea for ordering within the probationary tier seems more promising and is explicitly noted as untested in the cross-cycle insights. Let me explore that direction instead.

Looking back at the open questions: FNV-1a at 87.5% will be H01, frequency sketch for within-probationary ordering will be H02, and I'm reconsidering what H03 should be. The ceiling is confirmed at 87.5% protected (capacity/8 probationary), but I'm wondering if I should explore the opposite direction—what if I decreased the protected quota instead, like capacity/4 probationary for 75% protected?

Actually, that wouldn't work because the winners are already at 87.5% protected, so moving toward less protection would just reduce hit-rate. What if I borrowed from ARC instead—maintaining separate tiers for single-access and multi-access items with ghost lists to dynamically adapt the split? Though the cross-cycle insights suggest adaptive mechanisms fail on stationary workloads since the hot set is stable.

So for H03, I'm thinking about implementing a protected queue victim selection based on access frequency rank—instead of standard 2Q demotion that evicts the LRU item, I'd add a second-chance pass that skips items accessed multiple times recently.

Examining the Node structure... Looking at the size breakdown more carefully—if the Node is actually 16 bytes total, then it can't be storing both the key and value alongside the links. The most likely explanation is that the Node only holds the links array (8 bytes) plus the key (8 bytes for u64), while the value lives separately in the HashMap, keyed by the node index. This would explain the discrepancy I was seeing. So the Node struct would only hold the key and the prev/next links as u32 indices, keeping it to exactly 16 bytes, while the HashMap stores the actual values paired with their node indices. This separates the linked-list structure from the value storage, which aligns with that "link-array split" concept.

I could theoretically use the MSB of the next-pointer field to encode a second-chance bit since indices are bounded well below u32 max, but that's risky for correctness and probably not worth the complexity.

Let me refocus on what actually matters for this cycle—the FNV-1a hash function from cycle 0007 is a clear win on both per-operation time and struct size, and it's explicitly recommended, so that's a must-have.

For a second direction, I'm looking at frequency-ordered eviction from the probationary queue—it targets hit-rate using a compact frequency sketch as an ordering tool rather than a graduation gate, and it's an untested direction that was flagged in the cross-cycle questions.

For the third direction, I'm weighing whether to explore reducing the probationary quota below capacity/8 to see if I can maintain zero latency while improving hit-rate, though quota exploration has historically regressed above 87.5%. Shifting focus to HashMap load factor tuning—the default 0.875 fill ratio could be reduced by pre-allocating extra capacity to lower collision rates and improve get latency. But the incumbents are already hitting the measurement floor at per-op-time=0, so this optimization wouldn't move the needle further.

Instead, I'm considering a ghost cache approach: maintain a secondary HashMap tracking keys recently evicted from the probationary queue. When a put() arrives for a ghost key, bypass probationary entirely and admit directly to protected—this helps warm items that were dropped due to capacity pressure re-enter the protected tier immediately on re-access. This differs from two-hit admission because it's reactive rather than forward-looking. Admitting evicted items directly back to the protected tier could help with cycling behavior near the hot set boundary, but on workload-a the Zipfian distribution already keeps the hot set well-protected at 87.5%, so evicted items are likely genuinely cold. A ghost cache would need to be large enough to meaningfully capture warm-but-not-hot oscillation, which seems uncertain.

Instead, I'm considering a simpler approach: when a key first enters the cache, place it at a position in the probationary queue proportional to its estimated rank rather than at the tail, giving new items a better chance before immediate eviction.

But that requires frequency counting we don't have. Actually, maybe just placing new items at the MRU end of probationary instead of treating it as pure FIFO would help—though in a standard 2Q implementation, new items might already go to the MRU end by default. Let me check what the actual current implementation does with the protected and probationary queues.

In the standard 2Q setup, probationary has a max size of capacity/8, and when it overflows, the LRU item gets evicted. New items go to the probationary tail, and on a hit they get promoted to the protected tail. Protected uses LRU ordering while probationary uses FIFO. The quota tuning has already maximized hit-rate within this structure, so to improve further I need to change how eviction works within the queues themselves—specifically, I'm considering frequency-biased eviction from probationary.

Instead of always evicting the oldest item when probationary fills up, I can scan a small window of candidates from the head and evict whichever has the lowest access frequency. I'm using a compact 2-bit counter array with 256 entries to track frequency estimates, incrementing the counter for a key on both puts and gets. When eviction is needed, I scan 4 candidates, find the one with the minimum counter value, and evict that—breaking ties by choosing the oldest. The main concern is the branching overhead this introduces.

Now I'm formulating three distinct hypotheses to test. The first replaces the default hasher with FNV-1a to reduce struct size and per-operation time. The second introduces the frequency-biased eviction strategy using a 256-entry counter array to improve hit rates.

For the third hypothesis, I'm reconsidering whether pre-allocation actually helps since per-op-time is already optimal. Instead, I'm exploring a scan-resistant protected queue that splits into two implicit tiers, keeping the most recently accessed items in a hot tier to better resist sequential scans.

But I'm realizing this approach might not actually help on Zipfian workloads where the absolute LRU items in the protected queue are the right candidates for demotion anyway. I'm also considering whether expanding the probationary capacity could improve hit-rate despite the latency tradeoff, though I suspect the current 87.5% protected split is already optimal.

Looking at what's actually achievable: the hit-rate ceiling seems stuck around 0.732, per-operation time is already at the floor, and metadata size is at 16 bytes. The only meaningful improvements would be breaking through that hit-rate ceiling, reducing per-op time further (impossible), or fundamentally restructuring the node format to shrink metadata below 16 bytes.

For the metadata angle, I'm exploring whether we can compress the Node structure—the current 8-byte key plus 8 bytes for two u32 link indices hits 16 bytes. Reducing link indices to u16 doesn't help due to alignment padding, and using u8 indices only works for tiny caches. The real constraint is that we need both the key and the links, and alignment rules force us back to 16 bytes anyway.

The more promising direction is rethinking the eviction policy itself. Instead of the eager demotion in standard 2Q—where a probationary hit immediately promotes to protected and triggers immediate demotion of the LRU protected item if space is tight—I could use lazy promotion. When a probationary hit would overflow protected, I'd mark the protected-LRU item as pending demotion rather than moving it immediately, keeping it in place until it's actually needed.

This defers the expensive demotion work (unlinking and relinking) from the get path to the put path, where latency is less critical. The get path only sets a flag, which is nearly free. This could let me tighten the quota beyond 87.5% without regressing get latency, though put operations would see a slight increase.

The approach is actually a known scan-resistant technique: I'd add a pending-demotion flag to each node, and when a protected item gets a hit while the protected queue is full, I mark it for demotion instead of demoting immediately. If it gets another hit before eviction, the flag clears and it stays protected—giving hot items a second chance. During put or eviction, any pending-demotion item becomes the victim.

For the Node structure, I can use bit 31 of the `next` link field as this flag since link indices don't need that high bit, keeping the node size at 16 bytes.

The implementation strategy is: when a probationary hit would overflow the protected queue, mark the protected-LRU node with the demotion flag instead of immediately moving it; optionally give it a second chance if a promoted item hits it; then on eviction, process any flagged nodes first before evicting probationary ones. This buys time for protected items to prove their worth before actual demotion.

I'm reconsidering the quota adjustment though—the cycle analysis suggests 87.5% as the permanent ceiling, so pushing to 90% protected contradicts that guidance. Better to keep the quota at capacity/8 and just test whether the lazy demotion mechanism itself reduces latency, though at this point the get path is already at the measurement floor so gains might be imperceptible.

Since the incumbents already hit zero per-op time at 87.5% quota, lazy demotion wouldn't show measurable improvement there. I should probably pivot H03 toward hit-rate improvements through a different mechanism that doesn't require quota changes, and instead focus on the truly untested directions like FNV-1a hashing, cache-line alignment optimizations, or frequency-based eviction ordering within the probationary segment.

For H03 specifically, I'm exploring whether we can optimize the eviction path by reducing key clones or HashMap lookups—maybe using arena indices to make eviction more direct rather than requiring a full key-based removal from the HashMap.

But I'm also considering a different angle: whether the Node struct itself has unnecessary metadata overhead. If Node<K,V> is storing V when it shouldn't be, or if there's phantom data bloat, that could add up across the arena. The FnvBuildHasher swap already saved 16 bytes by removing the RandomState seed keys from the LruCache struct itself, so there might be similar wins in the node representation. reduce LruCache from 144 to 128 bytes through the FNV change, which improves per-operation time without affecting the metadata-size score since that dimension only measures Node bytes. To actually improve the score itself, I'd need to shrink Node below 16 bytes, which is challenging—using u16 indices for the links could theoretically get it to 12 bytes, but alignment padding would likely keep it at 16 anyway. Looks like 16 bytes is the practical minimum for the node structure due to alignment constraints, so I'll shift focus to optimizing hit rates and operation latency instead. For the protected queue, I'm exploring a scan-resistant approach where overflow eviction checks the two oldest items and demotes whichever one was accessed less recently, rather than blindly removing the absolute LRU.

Actually, wait—in a standard LRU queue, the tail is already the least recently accessed item, so demoting based on access recency is what we're already doing. Tracking admission time separately would add complexity without clear benefit over the existing LRU ordering.

I keep circling back to the same ideas. Let me narrow down to three concrete hypotheses worth testing: switching to FNV-1a hashing to reduce per-operation time and struct overhead, adding a small frequency counter in the probationary queue to improve hit-rate through frequency-biased eviction, and implementing lazy demotion with a higher quota to boost hit-rate while minimizing latency impact on the get path.

For H03 specifically, the key insight is that lazy demotion changes the mechanism itself—instead of eagerly demoting on every get that would overflow the protected region, I flag the protected tail node and defer the actual demotion until the next put operation. This avoids the per-operation cost that plagued earlier quota attempts. I'm using a bit flag stolen from the MSB of the next pointer in the protected-LRU node, which is safe since capacity is nowhere near 2^31, and when a get promotes from probationary and would overflow, I just set that flag in O(1) time rather than shuffling pointers around.

Implementing the lazy demotion... H03 should focus on the structural policy redesign that the cycle insight recommends, not quota increases above 87.5% which were explicitly ruled out. The challenge is ensuring that lazy demotion with a higher quota still fundamentally changes the mechanism enough to avoid revisiting the same rejected approaches.

Now I'm formalizing the three hypotheses, starting with H01 which replaces the default hasher with FNV-1a on the baseline to reduce struct size and hashing overhead.

Defining the FNV-1a hasher implementation...

Then I need to update the `LruCache` struct to use this custom hasher in its HashMap, modify the constructor to pass the hasher when creating the map, and keep the 87.5% protected quota unchanged while leaving all the promotion and eviction logic as-is. Next, I'm looking at a frequency-biased approach using a compact 256-entry frequency counter array to improve eviction decisions. On every `get()` or `put()` call that hits an existing key, I increment the frequency counter for that key's hash bucket, capping it at 15. For new insertions, I initialize the frequency to 1. When the probationary queue is full and needs to evict, I scan the 4 oldest candidates and remove whichever has the lowest frequency, using position as a tiebreaker. The protected queue and quota logic stay the same.

Now I'm deferring the protected-to-probationary demotion from the hot `get()` path to the `put()` eviction path using a pending-demotion flag, which lets me tighten the probationary quota to 10% of capacity instead of the current 12.5%. I'm using bit 31 of the `next` pointer field as the flag and defining accessor functions to extract the actual pointer and check the demotion status. Finalizing deferred demotions in the put path by checking the protected-queue head for the flag, then unlinking and moving the node to probationary or evicting it directly. Masking the demotion flag whenever reading a node's next index to get the true pointer, and maintaining prot_len bookkeeping so the protected queue can temporarily exceed capacity by one item during the deferral window.

Now verifying the hypothesis structure—H01's parent is cycle/0007/v002, and all three hypotheses cite cross-cycle winners from the incumbent set, which satisfies the requirement. They're differentiated by their focus areas.

Before writing the implementation files, I'm thinking through the practical constraints. H02's frequency-biased eviction needs to scan the four oldest probationary candidates, which means traversing the doubly-linked list—a small bounded cost of a few extra node reads. H03's lazy demotion mechanism is trickier and needs careful specification in the Changes section so the implementer understands the flow.

There's a subtle issue with H03 though: if multiple probationary hits trigger demotions before the protected queue can finalize them, we could end up with cascading pending-demotion flags across multiple nodes. Since each node only has one flag bit, I should constrain this to at most one pending demotion at a time. If a second demotion request arrives before the first is finalized, I'll force the first one to complete immediately on the get path rather than deferring it further.

This keeps the optimization for the common case—most promotions at 90% quota will be the first one, so deferral usually works—while guaranteeing correctness under contention. Let me update H03 with this refinement and verify that the parent variant cycle/0007/v002 is valid, then write out the files to the output directory.

### 09:04:26 chat

Now I have all the context I need. Let me synthesize the three hypotheses:

**Key findings:**
- All three cycle 0009 variants tried to exceed the 87.5% protected quota (capacity/8 probationary) and failed — the quota-latency coupling is confirmed structural
- FNV-1a hasher was only ever tested **paired with a quota increase** (cycle/0009/v003) — never in isolation at the clean 87.5% baseline. It reduced hashing cost and LruCache struct by 16 bytes but couldn't offset the demotion overhead at 93.75%
- Frequency sketch as **within-probationary eviction ordering** (not as a graduation gate) is explicitly listed as untested and open
- Lazy demotion (moving protected→probationary work from get to put path) is listed as an untested structural alternative to break the quota-latency coupling

I'll write three distinct hypotheses:
- **H01**: FNV-1a at clean 87.5% baseline (per-op-time + LruCache size target) 
- **H02**: Frequency-biased eviction ordering within probationary (hit-rate target)
- **H03**: Lazy deferred demotion + capacity/10 quota (hit-rate target via structural mechanism change)

### 09:04:28 tool: Bash

```json
{
  "command": "dir \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0010\\hypotheses\" 2>&1 || echo \"Directory does not exist yet\"",
  "description": "Check if output directory exists"
}
```

### 09:04:52 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0010\\hypotheses\\H01.md",
  "content": "# H01\n\nReplacing the default SipHash `RandomState` with a zero-sized FNV-1a `BuildHasher` on the cycle/0007/v002 baseline reduces both `LruCache` struct size and per-op hashing cost without altering the protected-queue policy or hit-rate.\n\n## Parent\n\ncycle/0007/v002\n\n## Changes\n\nThis is the \"FNV-1a in isolation\" experiment that cycle 0009 v003 confirmed works but only ever tested paired with a quota increase to capacity/16 (which regressed latency). Apply the FNV-1a hasher change alone, keeping `prob_cap = (capacity / 8).max(1)` unchanged.\n\n**Implementation steps:**\n\n1. At the top of `src/lru_cache/mod.rs`, add the following two zero-sized types (no external crate needed — implement FNV-1a inline):\n\n   ```rust\n   use std::hash::{BuildHasher, Hasher};\n\n   #[derive(Clone, Default)]\n   struct FnvBuildHasher;\n\n   impl BuildHasher for FnvBuildHasher {\n       type Hasher = FnvHasher;\n       fn build_hasher(&self) -> FnvHasher {\n           FnvHasher(0xcbf2_9ce4_8422_2325)\n       }\n   }\n\n   struct FnvHasher(u64);\n\n   impl Hasher for FnvHasher {\n       fn write(&mut self, bytes: &[u8]) {\n           for &b in bytes {\n               self.0 ^= b as u64;\n               self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);\n           }\n       }\n       fn finish(&self) -> u64 { self.0 }\n   }\n   ```\n\n2. Change the `map` field declaration in `LruCache<K, V>` from `HashMap<K, …>` (with default `RandomState`) to `HashMap<K, …, FnvBuildHasher>`, matching whatever value type the incumbent uses (e.g., `HashMap<K, (V, u32), FnvBuildHasher>`).\n\n3. In `LruCache::new()` (and any `with_capacity` constructor), replace `HashMap::with_capacity(capacity)` with `HashMap::with_capacity_and_hasher(capacity, FnvBuildHasher)`.\n\n4. Make no other changes: keep `prob_cap = (capacity / 8).max(1)`, the Node layout, the promotion/demotion logic, and the eviction path exactly as in cycle/0007/v002.\n\n**Expected effect**: `FnvBuildHasher` is a zero-sized type (no seed fields), so it removes 16 bytes from the `LruCache` struct relative to `RandomState`'s two-`u64` seed storage. FNV-1a also has lower per-byte hash computation cost than SipHash-1-3, which should either lower or preserve per-op-time. Node size stays at 16 bytes. Hit-rate is unchanged (hash function does not affect eviction policy).\n\n## Rationale\n\nCycle 0009 v003 confirmed that FNV-1a delivers a genuine struct-size benefit and lower hash cost, but those gains were masked by the simultaneous quota increase to 93.75% which dominated latency. At 87.5% (the confirmed zero-latency-cost ceiling), there is no demotion overhead to mask the hasher benefit. This is the cleanest remaining structural win that has not been directly measured.\n"
}
```

### 09:05:18 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0010\\hypotheses\\H02.md",
  "content": "# H02\n\nMaintaining a compact 256-entry frequency counter array in `LruCache` and using it to select the least-frequent among the four oldest probationary candidates for eviction improves hit-rate by preferentially retaining warm items over cold arrivals within the probationary queue.\n\n## Parent\n\ncycle/0007/v002\n\n## Changes\n\nThis hypothesis tests frequency-biased eviction **ordering within the probationary queue** — not as a graduation gate (which has been conclusively closed) and not as a filter on admission. The mechanism only changes *which* probationary item is evicted when the probationary queue overflows; it does not change the protected-queue policy, the quota, or the promotion path.\n\n**Implementation steps:**\n\n1. Add a field `freq: [u8; 256]` to the `LruCache<K, V>` struct. Initialise it to `[0u8; 256]` in `LruCache::new()`. This adds 256 bytes to the `LruCache` struct but does not affect `Node` size (metadata-size fitness dimension is unchanged).\n\n2. Define a cheap local frequency-index function that maps a key to one of 256 buckets without invoking the full `HashMap` hasher. For `K: Hash`, apply a single multiply-shift step on the key's raw bits:\n\n   ```rust\n   fn freq_bucket<K: std::hash::Hash>(key: &K) -> usize {\n       let mut h = std::collections::hash_map::DefaultHasher::new();\n       key.hash(&mut h);\n       let bits = std::hash::Hasher::finish(&h);\n       (bits.wrapping_mul(0x9e3779b97f4a7c15) >> 56) as usize\n   }\n   ```\n\n   Alternatively, if the key type is known to be `u64`, use `((key.wrapping_mul(0x9e3779b97f4a7c15)) >> 56) as usize` directly for zero overhead.\n\n3. **Increment counter on every access**: in `get()`, after confirming a cache hit (whether probationary or protected), increment `self.freq[freq_bucket(&key)]`, saturating at 255.\n\n4. **Initialise counter on new insertion**: in `put()` for a cache-miss (new key), set `self.freq[freq_bucket(&key)] = 1` before linking the new node into the probationary queue.\n\n5. **Frequency-biased eviction**: when the probationary queue must evict (probationary is full and a new item is being inserted), instead of always evicting the probationary head (oldest/LRU item), scan the **4 oldest** probationary nodes (the head and the next 3 nodes following `next` pointers from the LRU end). Among these 4 candidates, evict the one whose `freq[freq_bucket(&node.key)]` is smallest. Break ties by selecting the oldest (head-most) candidate. Unlink and evict that node, then insert the new item at the probationary MRU tail as normal.\n\n   If probationary has fewer than 4 items, scan only what is available.\n\n6. Keep `prob_cap = (capacity / 8).max(1)`, the Node layout (16 bytes), the protected-queue LRU order, the promotion/demotion logic, and all other paths exactly as in cycle/0007/v002.\n\n**Invariants to preserve**: the probationary and protected queue doubly-linked list structure must remain consistent after evicting a non-head candidate (i.e., properly unlink the chosen node by updating its predecessor's and successor's `links` fields). The frequency counter array is a best-effort heuristic; counter aliasing (multiple keys mapping to the same bucket) is acceptable — the mechanism degrades gracefully to LRU order when all candidates share the same counter value.\n\n## Rationale\n\nThe cross-cycle open questions explicitly identify \"frequency sketch as within-probationary eviction ordering\" as untested. Previous sketch-based attempts (cycle 0007 CountMin sketch as a graduation gate) failed because workload-a does not generate sufficient repeat accesses within a single window; however, using the sketch for *eviction ordering* rather than *admission filtering* is structurally different: every item in the probationary queue was already admitted (no gate), and the sketch only biases which of a small fixed-size candidate set is selected for removal. The bounded 4-item scan keeps the overhead constant and avoids the full-queue traversal cost that defeated earlier sweep-based approaches.\n"
}
```

### 09:05:50 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0010\\hypotheses\\H03.md",
  "content": "# H03\n\nDeferring the protected→probationary demotion from the `get()` hot path to the `put()` eviction path via a single-bit pending-demotion flag encoded in the existing `Node.links[1]` field allows the protected quota to be tightened to 90% (capacity/10 probationary) without regressing get p50 latency.\n\n## Parent\n\ncycle/0007/v002\n\n## Changes\n\nThis is a **structural mechanism change**, not a quota-only change. Previous cycle 0007–0009 attempts to exceed 87.5% all failed because the eager demotion sequence (unlink protected-LRU, insert into probationary tail) executed on the `get()` hot path every time a probationary hit promoted an item and the protected queue overflowed. This hypothesis decouples the quota from the per-get cost by deferring that work to the `put()` path, which already pays eviction costs.\n\n**Implementation steps:**\n\n1. **Steal bit 31 of `links[1]` as a demotion-pending flag.** Define at module scope:\n\n   ```rust\n   const DEMOTION_FLAG: u32 = 1 << 31;\n   ```\n\n   Define two inline helpers that mask the flag away when reading the actual index:\n\n   ```rust\n   #[inline(always)]\n   fn next_idx(node: &Node<K, V>) -> u32 { node.links[1] & !DEMOTION_FLAG }\n\n   #[inline(always)]\n   fn is_pending_demotion(node: &Node<K, V>) -> bool { node.links[1] & DEMOTION_FLAG != 0 }\n   ```\n\n   Update every read of `node.links[1]` as an index throughout the file to call `next_idx(node)` instead of `node.links[1]` directly. Capacity values in the benchmark suite are at most a few thousand, so bit 31 is always zero in the original code; stealing it is safe.\n\n2. **Change the quota**: set `prob_cap = (capacity / 10).max(1)` (90% protected, 10% probationary). This is the only quota change; no other policy parameters are modified.\n\n3. **Modify `get()` — lazy demotion**: when a probationary hit would cause the protected queue to overflow (i.e., `prot_len >= prot_cap` before the promotion step), check whether the protected queue's LRU tail already has `DEMOTION_FLAG` set:\n   - **If no pending demotion exists** (flag is clear on the protected-LRU tail): instead of executing the full demotion sequence, simply set `DEMOTION_FLAG` on that node's `links[1]` field (one bitwise-OR, no pointer manipulation) and proceed with the promotion of the probationary item into the protected queue. Increment `prot_len` by 1 for the newly promoted item; do NOT decrement it for the deferred demotion — the protected queue now transiently holds `prot_cap + 1` items with one flagged.\n   - **If a pending demotion already exists** (the protected-LRU tail already has the flag): fall back to immediate eager demotion — execute the full unlink-from-protected, push-to-probationary-tail sequence synchronously on the get path. This ensures at most one item at a time carries the flag, bounding the transient over-capacity state to a single node.\n\n4. **Modify `put()` — deferred demotion finalisation**: at the beginning of the eviction logic in `put()` (before selecting a probationary victim), check whether the protected queue's current LRU tail has `DEMOTION_FLAG` set. If so, finalise the deferred demotion now: clear the flag, unlink the node from the protected queue (update its predecessor's and successor's links), decrement `prot_len`, and push the node to the probationary MRU tail (incrementing the probationary size and triggering a further probationary eviction if that overflows). This moves the linked-list manipulation cost from `get()` to `put()`.\n\n5. **Bookkeeping invariant**: `prot_len` is the count of nodes currently linked in the protected queue, including any with `DEMOTION_FLAG`. After lazy demotion is triggered, `prot_len == prot_cap + 1` transiently. After finalisation on the `put()` path, it returns to `prot_cap`. Code that checks `prot_len >= prot_cap` must be reviewed to ensure the transient over-cap state does not trigger a second lazy demotion for the same item.\n\n6. **Node layout**: no change to `Node` size — the flag reuses a bit within the existing `links[1]` field. `Node<u64,u64>` remains 16 bytes.\n\n7. Leave the Node layout, the FNV-vs-SipHash choice, the probationary eviction policy (strict LRU/FIFO head eviction), and all other paths as in cycle/0007/v002.\n\n## Rationale\n\nCycles 0007, 0008, and 0009 confirmed from three independent angles (intermediate quota, demotion batching, FNV-1a offset) that the quota-latency coupling at the current policy is structural: any demotion work on the `get()` hot path shows up immediately in get p50. The cycle 0009 insight explicitly asks whether a policy redesign that avoids \"eager per-operation demotion enforcement\" could break this coupling. This hypothesis answers that question directly: by paying a single bit-OR on the `get()` path and deferring the full pointer manipulation to `put()`, we decouple the protected quota from the per-get cost. At capacity/10 (90% protected), the expected hit-rate improvement from holding more items in the hot tier should surface without the get-latency penalty that defeated all previous quota attempts.\n"
}
```

### 09:06:04 chat

All three hypotheses have been written to the output directory. Here's a summary of the cycle 0010 cohort:

---

### H01 — FNV-1a hasher at clean 87.5% baseline
**Parent**: `cycle/0007/v002` | **Target**: per-op-time + LruCache struct size

The FNV-1a hasher was validated in cycle 0009/v003 (removed 16 bytes from `LruCache`, lowered hash cost) but was only ever tested paired with a quota increase that then dominated latency. This hypothesis isolates the hasher change at the confirmed zero-latency-cost quota (capacity/8), letting its benefits surface without interference. A pure structural improvement with no behavioral risk.

---

### H02 — Frequency-biased eviction ordering within probationary
**Parent**: `cycle/0007/v002` | **Target**: hit-rate

Adds a compact 256-entry frequency counter array to `LruCache`. On probationary overflow, scans the 4 oldest candidates and evicts the one with the lowest access frequency rather than always the absolute FIFO-head. This is distinct from the conclusively-closed two-hit admission gate: it doesn't filter admissions, only biases *which already-admitted* item gets ejected. The 4-item bounded scan keeps overhead constant.

---

### H03 — Lazy deferred demotion with 90% protected quota
**Parent**: `cycle/0007/v002` | **Target**: hit-rate (structural mechanism change)

Steals bit 31 of `Node.links[1]` as a pending-demotion flag. When a `get()` probationary-hit would overflow the protected queue, instead of executing the full unlink+relink sequence on the hot path, a single bit-OR defers it to the next `put()` call. This decouples the protected quota from per-get cost — allowing `prob_cap = capacity/10` (90% protected) where three prior cycles failed with eager demotion. At most one deferred demotion is allowed at a time; a second overflow falls back to eager behavior for correctness.


---

- finished: 2026-07-21T09:06:05+00:00
- duration: 621.82s
- status: ok
- input_tokens: 7
- output_tokens: 19812
- cache_read_tokens: 186196
- cache_write_tokens: 31913
- cost_usd: 0.933242
