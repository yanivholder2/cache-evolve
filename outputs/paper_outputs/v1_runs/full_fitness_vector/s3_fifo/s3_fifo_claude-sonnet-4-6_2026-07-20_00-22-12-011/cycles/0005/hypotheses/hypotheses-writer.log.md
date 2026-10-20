# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0005\hypotheses
- allow_edits: True
- started: 2026-07-19T22:13:49+00:00

## timeline

### 22:13:54 thinking

Let me read the context files to understand the current state of the research and then generate 3 distinct hypotheses for cycle 0005.

Let me start by reading the key files in parallel.

### 22:13:55 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\cycle-insight.md"
}
```

### 22:13:55 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\cycle-winners.json"
}
```

### 22:13:55 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\leaderboard.md"
}
```

### 22:13:56 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cross-cycle-insights.md"
}
```

### 22:13:58 thinking

Now let me read the cycle 4 variants to understand their overlay implementations in more detail:

### 22:14:00 tool: Glob

```json
{
  "pattern": "v*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\variants"
}
```

### 22:14:01 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles-ledger.jsonl"
}
```

### 22:14:04 thinking

Now let me read the overlay files for cycle 0004 variants to understand what they looked like, and then read the baseline to understand the source code structure.

### 22:14:05 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\variants\\v001\\attempts\\01\\overlay"
}
```

### 22:14:06 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\variants\\v002\\attempts\\01\\overlay"
}
```

### 22:14:06 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\variants\\v003\\attempts\\01\\overlay"
}
```

### 22:14:09 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\variants\\v002\\attempts\\01\\overlay\\src\\s3_fifo\\mod.rs"
}
```

### 22:14:09 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\variants\\v001\\attempts\\01\\overlay\\src\\s3_fifo\\mod.rs"
}
```

### 22:14:12 thinking

Now let me read the baseline source code to understand the full structure before proposing hypotheses:

### 22:14:13 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\baseline\\src\\s3_fifo\\mod.rs"
}
```

### 22:14:14 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\variants\\v001\\eval\\fitness-vector.json"
}
```

### 22:14:18 thinking

Let me read the baseline directory to understand what files exist:

### 22:14:19 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\baseline"
}
```

### 22:14:22 thinking

The baseline directory doesn't seem to have the source code - it appears to be inputs with just eval data. Let me look for the actual mod.rs in the baseline subject:

### 22:14:22 tool: Glob

```json
{
  "pattern": "**/*.rs",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo"
}
```

### 22:14:23 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\variants\\v002\\insight.md"
}
```

### 22:14:27 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\src\\s3_fifo\\mod.rs"
}
```

### 22:20:18 thinking

Now I have a thorough understanding of the codebase and the research history. Let me synthesize what I know:

## Current state (cycle 0004 incumbent: v002)

The best performing variant is `cycle/0004/v002`, which combines:
1. `small_size = capacity/40` (smallest admission queue)
2. Parallel freq deques (freq out of Entry, in sibling VecDeques)
3. O(1) ghost membership + lazy deletion (ghost_set HashSet)

Current bottleneck: The **`position()` linear scan** in `get()` and `put()` — both search through `small` and `main` sequentially to find a key. 

The real problem is that scanning the main deque, which holds roughly 97.5% of capacity (around 2000 entries at capacity 2048), dominates the per-operation latency. The research across cycles consistently identifies this as the next critical target, suggesting solutions like intrusive node pointers, a side hashmap for deque lookups, or an index structure to achieve O(1) access.

I'm looking at how both `get()` and `put()` rely on linear scans through the deques to find keys, and considering a side HashMap that maps each resident key to its queue location and index for constant-time lookups instead of iterating through the deque.

The issue is that VecDeque's ring-buffer implementation means indices shift whenever items are pushed or popped from the front, so maintaining a HashMap with index pointers becomes problematic — every operation that modifies the front of the deque would require updating all stored indices.

I'm reconsidering whether I even need to track indices at all. The `position()` scan is really just used to find and update entries during get and put operations, so maybe there's a way to avoid storing indices entirely and instead keep the HashMap focused on the actual data while using the deques purely for ordering.

The real bottleneck is that on a Zipfian workload with high hit rates—around 73% of accesses hit, and most of those land in the main queue—I'm doing O(n) scans repeatedly. Adding a resident set could help skip scans on misses, but hits still require finding the entry's position. I'm thinking about whether storing both the queue reference and index together in the HashMap could help, though that introduces its own invalidation challenges. Defining the data structures... Implementing the `get()` method to increment frequency and return the value, then considering how `put()` and eviction would work with this structure. The tradeoff is that while operations stay O(1), the design requires multiple HashMaps which significantly increases memory overhead compared to the current compact `Entry` struct.

Looking at the actual metadata-size measurement though — it's specifically the per-entry cost of the `Entry<K,V>` struct at 16 bytes. If I switch to storing just keys in the VecDeque and moving values into a separate HashMap, the `Entry` struct disappears entirely, which changes how metadata-size would even be calculated. + 8 = 16 bytes minimum. If I use a HashMap for O(1) lookups with a VecDeque to maintain FIFO ordering, I'd store just keys in the deque and values in the map, which could let me redefine Entry as just containing the key—potentially 8 bytes. But I'm not sure if the metadata-size measurement would still work correctly with that change. Let me step back and think about what structural changes are actually feasible and what hypotheses I should test.

The real bottleneck is that `position()` linear scan in `get()` and `put()` — two independent variants already identified this as the dominant latency source. The top priority now is exploring O(1) alternatives: intrusive node pointers, a side hashmap into the deque, or an index structure.

For this cycle, I'm focusing on three main approaches. The first is adding a side HashMap that maps keys to queue tags and indices for O(1) membership checks in `get()` and `put()`, though I'd need to handle the fact that indices shift when pushing to the front or popping from the back.

The challenge is that even with a resident map, I still need to retrieve the actual value and frequency from the deque. A separate value map could let me bypass the deque lookup entirely, but that means maintaining multiple data structures in sync. I could add a `resident_set` HashSet to check membership in O(1) without storing indices, which would speed up the miss-path in `put()` by avoiding two `position()` scans. But hits still require O(n) lookups for value access and frequency updates, and with a 73% hit rate on the Zipfian workload, that's where most of the cost lies — so this is only a partial optimization. HashMap<K, V>` to avoid duplication. I'm realizing that to get O(1) performance for both `get()` and `put()`, I'd need a value map alongside the resident set and frequency map, but that introduces storage overhead by duplicating the values already in the VecDeques. ecDeque with just keys, and look up the frequency in `freq_map` to decide whether to promote or ghost the entry. If I redefine `Entry<K,V>` as a simple wrapper around K instead of storing the full value, that shrinks the metadata size significantly — though I need to check how the benchmark binary measures `std::mem::size_of::<Entry<K,V>>()`. Let me focus on a cleaner approach: separating the values into their own `HashMap<K, V>` so lookups stay O(1) without scanning the deques.

The new layout would have `small` and `main` as `VecDeque<K>` (keys only), a `freq_map: HashMap<K, u8>` for O(1) frequency access instead of parallel deques, a `data: HashMap<K, V>` for the actual values, and keep the existing `ghost_set` and `ghost` deque. This eliminates the need for `position()` entirely — `get()` just checks the ghost set for a miss, does a `data.contains_key()` for a hit, and updates frequency accordingly.

For `evict_main()`, I'd pop from the main deque, look up the frequency in `freq_map`, then either re-insert or fully evict (removing from `data` and `freq_map`). But I need to reconsider what `Entry<K,V>` should look like now, especially since the metadata-size test likely checks the size of the Entry struct itself.

If I switch to storing just keys in the VecDeque and move the actual entries to a HashMap, I could keep `Entry<K,V>` as a lightweight struct—maybe just a PhantomData marker, though that feels wrong. The safer approach is to keep `Entry<K,V>` containing the key and value (16 bytes for u64+u64), store it in the HashMap, and use a separate `VecDeque<K>` for ordering.

But that creates duplication—the key exists both in the HashMap's own key and inside the Entry struct. The real issue is whether `Entry<K,V>` needs to exist at all if I'm not storing it in a VecDeque anymore, and whether the measurement harness can even evaluate it if the type disappears.

Since `evolve_metadata_size.rs` is in the protected gate-integrity surface, the harness will always try to compile against `Entry<K,V>`. If the variant doesn't define it, compilation fails. So I have to keep the struct, even if it's not actively used—I just need to make it smaller or change how it's utilized.

The real strategy is to keep `Entry<K,V>` in the VecDeques for ordering, but add side HashMaps for O(1) lookups: one for values and one for frequencies, so I can skip the expensive `position()` calls in `get()` and `put()`.

A cleaner approach is to redefine `Entry<K,V>` to hold only the key with a PhantomData marker for the value type, shrinking it to just 8 bytes, then use separate HashMaps to store the actual values and frequency data while the VecDeques maintain insertion order. I'm realizing the memory overhead gets more complex when accounting for HashMap internals — each HashMap entry carries its own per-entry cost (around 24 bytes for u64 keys), so the total footprint per cache slot becomes roughly 32 bytes (HashMap overhead plus the 8-byte Entry) rather than the 24 bytes we'd save by shrinking Entry alone.

But looking at how the fitness metric is actually calculated, there are two separate measurements: the Entry struct itself at 16 bytes and the full S3Fifo struct at 224 bytes, yet the summary only uses the Entry size of 16 as the reported metric. So adding more HashMaps to the S3Fifo control structure increases the overall struct size, but the fitness framework is only tracking the Entry size, not the full control overhead.

Now I'm considering three different hypotheses to improve performance. The first one targets per-operation time by replacing the position scanning logic with a side HashMap that stores values for resident keys, plus a frequency map to track access counts.

The core changes involve restructuring Entry to only hold the key (reducing it to 8 bytes), moving the actual values into the HashMap, and using direct lookups instead of scanning through deques. This means get() operations become O(1) for both value retrieval and frequency updates, and put() operations can check key existence instantly. The eviction logic would then...

Alternatively, I could take a simpler approach by adding a resident HashSet that tracks which keys are currently in the cache, letting me short-circuit the miss path in both get() and put() without a full restructure.

But I'm realizing this only helps with the 27% miss rate—the 73% hit path still needs position() to find and update values, so it's a partial win. The real bottleneck is that position() scan on hits, which this approach doesn't address. The 73% hit rate means most get() calls still need to scan the deques, so this optimization only helps the minority case. I'm considering a bigger restructure instead: replacing the deque of entries with separate deques for keys and hashmaps for values and frequencies. This would let me do O(1) lookups while keeping the ordering information, though H01 and H03 are starting to converge on similar approaches. Approach A uses a full side HashMap to store values alongside frequency tracking and key-only deques, eliminating all position() calls and shrinking Entry to 8 bytes but expanding the overall control structure. Approach B takes a lighter touch by adding just a HashSet for O(1) residence checks and a frequency map while keeping the deque-based structure mostly intact.

I'm realizing A and B converge on the same solution, so let me explore different angles. Approach C would use an ordered map like IndexMap that maintains both O(1) lookup and insertion order, but that requires an external crate which isn't available in this environment. Approach D attempts to track slots via indices that get updated, though the exact mechanism is still unclear.

Now I'm considering whether raw pointers into a VecDeque could work — storing HashMap entries that point directly to queue elements — but that's too fragile since VecDeque reallocations would invalidate those pointers. A more practical angle is Approach E: recognizing that on Zipfian distributions, a small fraction of keys dominate the access pattern, so I could maintain a "hot tier" with O(1) lookup for frequently accessed keys while keeping less-used entries elsewhere.

But this adds complexity and changes the eviction semantics significantly, which could hurt correctness. Let me step back and focus on the three most distinct and testable approaches: the full side-map solution with separate HashMaps for values and frequencies to achieve O(1) operations on both get and put.

The second approach is a narrower fix using just a resident_set HashSet to guard the miss path, eliminating the deque scan for the ~27% of misses while leaving the hit path unchanged. This is less intrusive and isolates whether the miss-path scanning is actually a bottleneck.

The third approach replaces the parallel frequency deques with a single HashMap keyed by entry, so instead of scanning deques to update frequencies, I can do direct lookups.

But I'm realizing the core issue: `position()` is the real bottleneck in both `get()` and `put()` for the hit path, and the only way to fix that is with a side HashMap that maps keys directly to values for O(1) retrieval. So I'm considering a full refactor that adds this data map alongside the existing deques.

The approach would be to strip down the Entry struct to just hold the key, using PhantomData for the value type, which keeps entries small. Then the deques only manage ordering while the HashMap handles actual value storage. This way `get()` becomes a simple O(1) lookup in the data map without ever calling `position()`, and `put()` updates can check and insert into the data map directly without searching through the deques.

For eviction, when popping entries from either deque, I'd look up their frequency in the frequency map and either promote them with a decremented frequency or remove them entirely from both the data map and frequency map. The ghost deque and set stay for tracking evicted keys, and I'd remove the parallel frequency deques since the single frequency map replaces them. All existing tests should still pass with these structural changes.

The second approach is more conservative—just adding a resident set that tracks which keys are currently in memory. On admission and eviction, I'd keep the set in sync. During gets, I can check the resident set first to skip the scan entirely on cache misses. During puts, I'd check residency to decide whether to scan for an update or go through the ghost logic for a new admission. This tests whether the miss-path scanning is the real bottleneck, expecting a more modest improvement.

For the third angle, I'm reconsidering the approach entirely. Building from scratch with all the previous wins would just recreate what's already in the incumbents. Instead, I'm looking at the performance data more carefully—gets are fast at 100ns p50, but puts are much slower at 700ns p50, with the bottleneck traced to the `position()` call during put operations.

When putting a ghost hit, the operations are all O(1), but for new keys, the algorithm scans both the small cache (~100 entries) and the main cache (~1950 entries) to check positions. That full scan of main at near-capacity is what's dragging put performance down compared to get, which doesn't need to do these lookups.

I'm considering a different approach: consolidate all the per-entry metadata into a single HashMap that maps each key to its queue location (small or main), its value, and its frequency. This eliminates the parallel frequency deques and the need to scan for positions entirely. Now get and put updates become simple HashMap lookups, and admitting new entries requires no scanning at all.

When evicting, I'd look up the frequency directly from this resident map to decide whether to promote from small to main or demote to ghost. The tradeoff is that I'm using one larger HashMap instead of two separate ones, but the memory layout might actually be better for cache locality on the common get path. I'm comparing three concrete hypotheses: H01 uses separate HashMaps for values and frequencies with key-only entries to eliminate position() calls entirely, while H02 adds just a resident HashSet for O(1) miss detection without removing position() from the hit path, and H03 consolidates value and frequency into a single HashMap entry for better cache locality. I'm weighing whether to use the same parent for all three hypotheses or branch them differently. Using `cycle/0004/v002` for all three risks redundancy between H02 and H03, so I'm considering deriving H03 from an earlier version like `cycle/0003/v002` instead, which already has the 1/20 small queue optimization as a foundation.

The key insight is that this setup would let me isolate the impact of the O(1) lookup fix itself: H01 would test it on top of the 1/40 small queue baseline, while H03 would test it on top of the 1/20 baseline. That comparison could reveal whether the O(1) lookup improvement is strong enough to salvage the 1/40 configuration or if 1/20 remains superior.

But the instructions push for truly distinct strategies rather than near-duplicates, so I need to think beyond just swapping HashMap for a different lookup structure. I'm exploring alternatives like using a raw index table, pre-sorting entries through the HashMap to create a fast lane, or building a frequency-ordered skip structure — though each of these either reintroduces complexity or doesn't cleanly solve the core problem. The real challenge is finding a fundamentally different approach that doesn't just tweak the lookup mechanism.

One angle is to make the ghost_set and resident tracking do more work together — maintaining both a resident_set and value_map for O(1) checks and value retrieval, though that's essentially H01 with a different structure. Another angle is to specifically target put() tail latency by adding a HashMap that tracks queue positions and lazily invalidates stale entries, focusing optimization on the bottleneck rather than the whole operation.

Looking at the core problem again, I'm settling on three distinct approaches that all target the position() bottleneck but through different mechanisms: H01 does a complete restructure with a data HashMap and freq_map to eliminate all position() calls entirely, while H02 takes an incremental approach with guards.

For H03, instead of just repeating H01's mechanism from a different ancestor, I should explore something genuinely different — maybe focusing on whether we can optimize the miss detection path separately from the hit path, or testing if a hybrid approach works better than going full O(1) everywhere.

Actually, I'm considering an alternative direction: what if we inline frequency tracking directly into the ghost and resident sets rather than maintaining parallel deques? We'd use a HashMap to track frequency only for entries that actually have freq > 0, since the second-chance logic in eviction only needs counts for those hot entries anyway. This could reduce memory overhead while still supporting the frequency-based promotion decisions.

But this still requires knowing whether a key is in main to update its frequency during a get(), which means we're back to needing some form of O(1) membership check. The real insight is that to avoid the position() lookup on cache hits, we'd need a separate HashMap storing the values themselves—there's no way around that without fundamentally changing the data structure.

Let me explore a few concrete variations on this approach. H01 and H03 both pursue full O(1) lookup via hashmap but differ in their memory layout—two separate maps versus a single merged map—which could affect cache behavior. H02 is a lighter partial fix that only guards the miss-path to isolate whether scanning is the real bottleneck. The overlay only modifies `src\s3_fifo\mod.rs`, while the JNI and library files remain protected.

Now I'm writing up the three hypothesis specifications, starting with H01 which adds separate HashMaps for values and frequencies to eliminate the linear scan in get and put operations. Refining the `get()` method to use O(1) lookups instead of iteration, then updating `put()` to handle the update, ghost-promotion, and new-admission paths with direct map operations, and finally adjusting both eviction functions to pop from their deques, fetch frequency from the map, and apply second-chance logic or removal accordingly.

I'm also preserving the small-size ratio, ghost queue and set, and lazy deletion invariants from the previous version, then introducing a `resident_set` guard to short-circuit misses in both `get()` and `put()` with O(1) membership checks. I need to add a guard check in `put()` to skip the two `position()` scans when a key isn't resident, letting it jump straight to the ghost-check and admission logic. I'm also considering a merged HashMap approach that stores both value and frequency together for O(1) lookups instead of scanning through deques.

Refactoring `Entry<K, V>` to hold only the key with a phantom type marker, removing the separate frequency deques entirely, and replacing them with a single `resident_map` that pairs each key with its value and frequency. In `get()`, I'd do a direct O(1) lookup to fetch the value and frequency, increment the frequency in place, and return — no deque scanning needed. The `put()` update path follows the same pattern with a single map lookup.

For ghost promotion, I'd check the ghost set and insert into `resident_map` with frequency 0. New admissions go straight into `resident_map` and push a key-only Entry to the small deque. When evicting from either deque, I'd pop the Entry, look up its frequency in `resident_map`, and either decrement and re-queue it if it's still hot, or remove it entirely if the frequency is exhausted.

I'm also reconsidering whether to keep the bounded scan cap from v001 — the cycle 4 analysis flagged it as falsified, so I might drop that optimization and rely on the frequency-based promotion instead. Unbounded second-chance is better since the scan cap was falsified, so H03 drops that limitation and uses a resident_map for O(1) lookups instead of the frequency arrays. Now I'm clarifying the three hypotheses: H01 is the full O(1) refactor from v002 with capacity/40 small size and HashMap-based frequency tracking, while H03 keeps v001's structure with capacity/20 but removes the scan cap and swaps in the resident_map. /40)
- Eliminates ALL position() calls
- Tests single-map vs two-map layout and 1/20 vs 1/40 interaction

These three hypotheses are distinct: H01 uses two maps with 1/40 sizing, H02 adds only a resident set without other changes, and H03 switches to a single map with 1/20 sizing. Now I'm going to write out the actual hypothesis implementations, starting with H01 where I need to carefully identify which functions change—specifically removing the position() calls from get() and put() in the small and main deques.

For H01, I'm replacing the frequency tracking deques with a HashMap-based approach and adding a data HashMap to store values, while restructuring the Entry type to remove the value field. Writing eviction logic...

I'm realizing there's a subtle issue with how ghost entries work—when a cold entry gets evicted to the ghost set, it needs to be removed from the data and frequency map since it's no longer resident, but the key stays in ghost. When that ghost entry gets a hit later, it gets re-inserted back into data and the frequency map, which should handle the lifecycle correctly.

The unit tests are going to need updates since they directly manipulate the frequency lists we're removing. Tests like the main eviction one access `small_freq` and `main_freq` directly, so those assertions will need to be rewritten. The ghost hit and small eviction promotion tests should still work since they just check if entries exist in the main list by key.

For the Entry constructor, I realize we have a choice: either change it to only take a key parameter, which would break all existing test code calling `Entry::new(1, 10)`, or keep the constructor signature but have it accept and ignore the value parameter. The second approach is cleaner for backward compatibility — the tests can keep calling `Entry::new(1, 10)` without modification, the value just gets discarded.

But there's a bigger issue: the white-box tests directly manipulate internal fields like `main_freq`, which won't exist anymore if we're removing the frequency tracking. Tests like `main_eviction_uses_frequency_second_chance` that push entries into `main` and then into `main_freq` will fail to compile. We need to update or replace these tests in the overlay module.

For the H02 changes with just the resident set, I need to add the `resident_set: HashSet<K>` field, initialize it in `try_new()`, and then handle the new admission logic in `put()`.

Adding resident set tracking... Updating the `get()` method to use a direct HashMap lookup instead of scanning the deques, and similarly refactoring `put()` to check the resident map first before falling back to ghost promotion — this eliminates the position-finding overhead and keeps the frequency tracking centralized in the resident_map. Keeping `small_size` at capacity/20 and preserving the ghost deque with lazy deletion logic. In `evict_main()`, I need to handle the case where an entry might not be in `resident_map` by using a safe lookup, and when decrementing frequency, I'll update the map in-place using `get_mut()` to avoid double lookups.

Now I'm ready to write out the hypothesis files with concrete claims about what changes target which fitness dimensions. I need to refine each claim to be more precise and testable. Let me make sure they each state a concrete change, name the fitness dimension they target, and fit in one sentence. H01 should specify replacing the parallel deques and position scan with separate data and frequency maps to eliminate O(n) lookups. H02 needs to clarify how the resident set guard short-circuits the double-deque traversal. H03 should emphasize consolidating both value and frequency into a single map to avoid metadata overhead.

Now I'm checking the parent requirement—at least one hypothesis must cite a cross-cycle winner, and I need to verify which versions qualify before assigning parents. angles. I need to write the hypothesis files to the output directory, making sure each one targets the position() bottleneck from a different approach. One thing to verify: PhantomData works fine with the key bounds I'm using, and I need to handle the test cases where Entry::new is called with a placeholder value.

I'm also checking which tests access internal structures directly — ghost_set and ghost still exist across all hypotheses, but tests that push directly into main_freq or small_freq will need updating to populate resident_map or freq_map instead. Looking at the main_eviction_uses_frequency_second_chance test, it's constructing entries and frequencies manually...

Updating test setup... Handling lifetime issues with mutable borrows...

The key insight is that `&mut V` coerces to `&V`, so I can destructure the mutable reference from `get_mut`, update the frequency counter, and return the value as an immutable reference — the lifetime ties back to `&mut self` which is valid. For the `get()` method on `H01`, borrowing `data` immutably within a `&mut self` context is also fine since `get()` doesn't require exclusive access. The borrow of `freq_map` ends after the mutation, so the subsequent call to `self.data.get(key)` creates a new non-overlapping borrow that should work fine with NLL. However, there's a subtlety with the return type—the `&V` reference from `self.data.get(key)` is tied to the lifetime of `&mut self`, and Rust will coerce the mutable borrow to an immutable one for that call.

For the H03 `get()` method, I'm returning a mutable reference coerced to an immutable one after incrementing the frequency counter. The borrow of `resident_map` technically remains alive through the returned reference, but this is valid since we're just returning it with a coerced lifetime.

Now I'm moving on to finalize the hypothesis files and examine the `put()` update path in H01, which handles both existing resident entries and ghost hits. Using the Entry API would let me do a single lookup instead of two operations, though it requires cloning the key even on a miss. For now, the straightforward `contains_key` + `insert` approach is clear and still O(1), which is what matters. Now I need to write out the final hypothesis files carefully, making sure that when an entry gets evicted with frequency zero, I remove it from both `data` and `freq_map` without adding it to the ghost set.

In `evict_small()`, entries with frequency greater than one get promoted to main while staying in the data structures, but entries with frequency one or less move to the ghost set. When a ghost hit occurs, I remove the key from the ghost set, evict from main if necessary, reinsert into `data` and `freq_map` with frequency zero, and push the key to the main deque.

For the `try_new()` initialization, I'm sizing both `data` and `freq_map` to the total capacity since they'll hold all resident entries, while `ghost_set` gets sized to just the main cache size.

Now I'm ready to write out the three files to the output directory, starting with verifying the path exists.

### 22:20:18 chat

Now I have all the context I need. Let me verify the output directory exists, then write the three hypothesis files.

### 22:20:18 tool: Bash

```json
{
  "command": "ls \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\hypotheses\" 2>/dev/null && echo \"exists\" || echo \"missing\"",
  "description": "Check if output hypotheses directory exists"
}
```

### 22:20:27 thinking

Good, the directory exists. Now let me write the three hypothesis files.

Let me compose them carefully:

**H01**: Full O(1) get/put via two side HashMaps (data + freq_map), starting from cycle/0004/v002 (best incumbent, 1/40 small).

**H02**: Minimal resident_set guard only, starting from cycle/0004/v002 (tests if miss-path scanning is significant).

**H03**: Single merged HashMap (resident_map: K→(V,u8)), starting from cycle/0004/v001 (1/20 small, tests single-map vs two-map and whether 1/20 works better with O(1) lookup).
Now I'll structure each file with a header, the claim, a parent section pointing to the baseline, and a changes section documenting what's different.

### 22:20:53 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\hypotheses\\H01.md",
  "content": "# H01\n\nReplacing the `position()` scan in `get()` and `put()` of `cycle/0004/v002` with a side `data: HashMap<K, V>` for O(1) value retrieval and a `freq_map: HashMap<K, u8>` for O(1) frequency access eliminates the dominant O(n) lookup bottleneck identified in cycle 4 and reduces per-op-time.\n\n## Parent\n\ncycle/0004/v002\n\n## Changes\n\nStarting from the `cycle/0004/v002` overlay (`src/s3_fifo/mod.rs`), apply the following changes. Every other aspect of v002 — `small_size = capacity/40`, ghost lazy-deletion via `ghost_set`/`ghost` deque, `MAX_FREQ = 3`, `evict_main()` unbounded second-chance loop, `evict_small()` freq>1 promotion threshold — remains intact.\n\n### 1. Redefine `Entry<K, V>` to hold only the key\n\nRemove the `value: V` field. Add `_phantom: std::marker::PhantomData<V>` so that `Entry<K,V>` remains generic over `V` without storing it. Keep the constructor signature `fn new(key: K, _value: V) -> Self` (accepting but ignoring the value argument so all existing call-sites compile unchanged). After this change `std::mem::size_of::<Entry<u64,u64>>()` = 8.\n\n```rust\n#[derive(Debug)]\npub struct Entry<K, V> {\n    key: K,\n    _phantom: std::marker::PhantomData<V>,\n}\n\nimpl<K, V> Entry<K, V> {\n    fn new(key: K, _value: V) -> Self {\n        Self { key, _phantom: std::marker::PhantomData }\n    }\n}\n```\n\n### 2. Add `data` and `freq_map` fields; remove parallel freq deques\n\nIn `S3Fifo<K, V>`, remove `small_freq: VecDeque<u8>` and `main_freq: VecDeque<u8>`. Add:\n\n```rust\ndata: HashMap<K, V>,      // resident key → value; O(1) access\nfreq_map: HashMap<K, u8>, // resident key → frequency; O(1) access\n```\n\nAdd `use std::collections::HashMap;` at the top. In `try_new()`, remove the `small_freq`/`main_freq` capacity allocations and add:\n\n```rust\ndata: HashMap::with_capacity(capacity),\nfreq_map: HashMap::with_capacity(capacity),\n```\n\n### 3. Rewrite `get()` to use O(1) lookups\n\nReplace the entire body of `get()`:\n\n```rust\npub fn get(&mut self, key: &K) -> Option<&V> {\n    if let Some(freq) = self.freq_map.get_mut(key) {\n        *freq = freq.saturating_add(1).min(MAX_FREQ);\n        return self.data.get(key);\n    }\n    None\n}\n```\n\nNo `position()` calls remain. The borrow of `freq_map` ends after the `*freq` assignment, so the subsequent `self.data.get(key)` borrow is disjoint and compiles under NLL.\n\n### 4. Rewrite `put()` to use O(1) lookups\n\nReplace the update-check and admission logic in `put()`:\n\n```rust\npub fn put(&mut self, key: K, value: V) {\n    // O(1) update path: key already resident.\n    if self.data.contains_key(&key) {\n        self.data.insert(key, value);\n        return;\n    }\n\n    // Ghost hit: O(1) via ghost_set.\n    if self.ghost_set.contains(&key) {\n        self.ghost_set.remove(&key);\n        if self.main.len() >= self.main_size {\n            self.evict_main();\n        }\n        self.data.insert(key.clone(), value);\n        self.freq_map.insert(key.clone(), 0);\n        self.main.push_front(Entry::new(key, value));  // value arg ignored\n        return;\n    }\n\n    // Normal admission into small.\n    if self.small.len() >= self.small_size {\n        self.evict_small();\n    }\n    self.data.insert(key.clone(), value);\n    self.freq_map.insert(key.clone(), 0);\n    self.small.push_front(Entry::new(key, value));  // value arg ignored\n}\n```\n\nNo `position()` calls remain.\n\n### 5. Rewrite `evict_main()` to use `freq_map`\n\n```rust\nfn evict_main(&mut self) {\n    while let Some(tail) = self.main.pop_back() {\n        let freq = self.freq_map.get(&tail.key).copied().unwrap_or(0);\n        if freq > 0 {\n            self.freq_map.insert(tail.key.clone(), freq - 1);\n            self.main.push_front(tail);\n        } else {\n            self.data.remove(&tail.key);\n            self.freq_map.remove(&tail.key);\n            break;\n        }\n    }\n}\n```\n\n### 6. Rewrite `evict_small()` to use `freq_map`\n\n```rust\nfn evict_small(&mut self) {\n    if let Some(tail) = self.small.pop_back() {\n        let freq = self.freq_map.get(&tail.key).copied().unwrap_or(0);\n        if freq > 1 {\n            // Hot: promote to main; entry stays in data and freq_map.\n            if self.main.len() >= self.main_size {\n                self.evict_main();\n            }\n            self.main.push_front(tail);\n        } else {\n            // Cold: evict to ghost; remove from data and freq_map.\n            self.data.remove(&tail.key);\n            self.freq_map.remove(&tail.key);\n            while self.ghost.len() >= self.main_size {\n                if let Some(evicted_key) = self.ghost.pop_back() {\n                    if self.ghost_set.contains(&evicted_key) {\n                        self.ghost_set.remove(&evicted_key);\n                        break;\n                    }\n                }\n            }\n            let k = tail.key;\n            self.ghost.push_front(k.clone());\n            self.ghost_set.insert(k);\n        }\n    }\n}\n```\n\n### 7. Update the white-box unit test\n\nThe test `main_eviction_uses_frequency_second_chance` directly pushes into `main_freq` (which no longer exists). Replace those pushes with equivalent `freq_map` and `data` insertions:\n\n```rust\ncache.main.push_back(Entry::new(1, 10)); cache.data.insert(1u64, 10u64); cache.freq_map.insert(1u64, 0u8);\ncache.main.push_back(Entry::new(2, 20)); cache.data.insert(2u64, 20u64); cache.freq_map.insert(2u64, 2u8);\ncache.main.push_back(Entry::new(3, 30)); cache.data.insert(3u64, 30u64); cache.freq_map.insert(3u64, 0u8);\ncache.main.push_back(Entry::new(4, 40)); cache.data.insert(4u64, 40u64); cache.freq_map.insert(4u64, 1u8);\ncache.small.push_front(Entry::new(5, 50)); cache.data.insert(5u64, 50u64); cache.freq_map.insert(5u64, 2u8);\n```\n\nAll other tests — `get_miss_returns_none`, `put_then_get_finds_value`, `put_existing_updates_value`, `small_eviction_promotes_to_main_after_multiple_hits`, `ghost_hit_promotes_back_to_main`, `capacity_bound_never_exceeded`, `repeated_get_clamps_frequency`, `ghost_set_len_bounded_after_ghost_hit` — use only the public API and require no changes.\n"
}
```

### 22:21:17 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\hypotheses\\H02.md",
  "content": "# H02\n\nAdding a `resident_set: HashSet<K>` membership guard to `cycle/0004/v002` short-circuits the double `position()` scan on cache misses in both `get()` and `put()`, reducing per-op-time for the miss path without restructuring the hit-path data layout.\n\n## Parent\n\ncycle/0004/v002\n\n## Changes\n\nStarting from the `cycle/0004/v002` overlay (`src/s3_fifo/mod.rs`), apply the following targeted changes. The `Entry<K,V>` struct, parallel `small_freq`/`main_freq` deques, `position()`-based hit path, and all eviction logic remain untouched. This hypothesis tests whether miss-path scanning (not hit-path scanning) is a material fraction of total per-op cost.\n\n### 1. Add `resident_set: HashSet<K>` field\n\nIn `S3Fifo<K,V>`, add:\n\n```rust\n/// Set of all keys currently resident in `small` or `main`. Maintained in\n/// lock-step with admissions and evictions to enable O(1) membership checks.\nresident_set: HashSet<K>,\n```\n\n`HashSet` is already imported via `use std::collections::HashSet;`. In `try_new()`, add:\n\n```rust\nresident_set: HashSet::with_capacity(capacity),\n```\n\n### 2. Maintain `resident_set` in `put()` on new admission\n\nIn `put()`, in the normal-admission branch (after the ghost-hit and update checks), insert the key before pushing to small:\n\n```rust\nself.resident_set.insert(key.clone());\nself.small.push_front(Entry::new(key, value));\nself.small_freq.push_front(0);\n```\n\nIn the ghost-hit branch, insert the key before pushing to main:\n\n```rust\nself.resident_set.insert(key.clone());\nself.main.push_front(Entry::new(key, value));\nself.main_freq.push_front(0);\n```\n\n### 3. Remove key from `resident_set` on eviction\n\nIn `evict_main()`, in the `freq == 0` (eviction) branch, before `break`:\n\n```rust\nself.resident_set.remove(&tail.key);\n```\n\nIn `evict_small()`, in the cold (ghost) branch, remove before pushing to ghost:\n\n```rust\nself.resident_set.remove(&tail.key);\n```\n\nDo NOT remove in the hot (promote) branch — the key moves from small to main and remains resident.\n\n### 4. Guard `get()` with an O(1) miss check\n\nInsert an early-return at the very top of `get()`, before the two `position()` calls:\n\n```rust\npub fn get(&mut self, key: &K) -> Option<&V> {\n    if !self.resident_set.contains(key) {\n        return None;  // O(1) miss: skip both position() scans\n    }\n    // Hot path: key is resident — use existing position()-based lookup.\n    if let Some(pos) = self.small.iter().position(|e| &e.key == key) {\n        let f = &mut self.small_freq[pos];\n        *f = f.saturating_add(1).min(MAX_FREQ);\n        return Some(&self.small[pos].value);\n    }\n    if let Some(pos) = self.main.iter().position(|e| &e.key == key) {\n        let f = &mut self.main_freq[pos];\n        *f = f.saturating_add(1).min(MAX_FREQ);\n        return Some(&self.main[pos].value);\n    }\n    None\n}\n```\n\nThe `resident_set.contains` is O(1); on a miss (~27% of accesses on this Zipfian workload) both `position()` scans are skipped entirely.\n\n### 5. Guard `put()` update path with an O(1) resident check\n\nReplace the two `position()` update checks at the top of `put()` with a guarded block:\n\n```rust\npub fn put(&mut self, key: K, value: V) {\n    // O(1) guard: only scan for the entry when key is known to be resident.\n    if self.resident_set.contains(&key) {\n        if let Some(pos) = self.small.iter().position(|e| e.key == key) {\n            self.small[pos].value = value;\n            return;\n        }\n        if let Some(pos) = self.main.iter().position(|e| e.key == key) {\n            self.main[pos].value = value;\n            return;\n        }\n    }\n    // ... ghost check and admission continue unchanged ...\n}\n```\n\nFor new keys (`resident_set.contains` = false), both `position()` scans are skipped and the code falls through directly to the ghost-check and admission path, eliminating O(n) work on every non-update `put()`.\n\n### 6. No changes to unit tests required\n\nAll existing tests exercise only the public API (`put`/`get`) and white-box fields that are unchanged (`small`, `main`, `ghost`, `ghost_set`, `small_freq`, `main_freq`). The `resident_set` field is internal bookkeeping and no test directly inspects it. Add the `resident_set` field access to `S3Fifo` without the `pub` modifier so it remains crate-private.\n"
}
```

### 22:21:54 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\hypotheses\\H03.md",
  "content": "# H03\n\nReplacing the `position()` scan in `cycle/0004/v001` with a single `resident_map: HashMap<K, (V, u8)>` that stores both value and frequency per resident key eliminates all O(n) lookups in `get()` and `put()` via one cache-local hashmap lookup instead of two separate maps, reducing per-op-time.\n\n## Parent\n\ncycle/0004/v001\n\n## Changes\n\nStarting from the `cycle/0004/v001` overlay (`src/s3_fifo/mod.rs`), which has `small_size = capacity/20`, parallel `small_freq`/`main_freq` deques, O(1) ghost via `ghost_set`, lazy ghost deletion, and a bounded `SCAN_CAP = 16` in `evict_main()`. Apply the following changes. `small_size = capacity/20` and all ghost logic are preserved; the bounded scan cap is removed (the cycle 4 insight showed it is falsified on Zipfian workloads).\n\n### 1. Redefine `Entry<K, V>` to hold only the key\n\nRemove the `value: V` field from `Entry<K,V>`. Keep the constructor accepting `(key: K, _value: V)` so all existing call-sites compile without modification. Add `_phantom: std::marker::PhantomData<V>` to retain the generic parameter. After this change `std::mem::size_of::<Entry<u64,u64>>()` = 8.\n\n```rust\n#[derive(Debug)]\npub struct Entry<K, V> {\n    key: K,\n    _phantom: std::marker::PhantomData<V>,\n}\n\nimpl<K, V> Entry<K, V> {\n    fn new(key: K, _value: V) -> Self {\n        Self { key, _phantom: std::marker::PhantomData }\n    }\n}\n```\n\n### 2. Replace parallel freq deques with `resident_map`; remove bounded scan constant\n\nIn `S3Fifo<K,V>`, remove `small_freq: VecDeque<u8>` and `main_freq: VecDeque<u8>`. Add:\n\n```rust\n/// Maps every resident key to its (value, frequency) pair. O(1) access for\n/// both get() value retrieval and frequency updates. A single lookup covers\n/// both fields, avoiding a second hashmap round-trip.\nresident_map: HashMap<K, (V, u8)>,\n```\n\nAdd `use std::collections::HashMap;` at the top. In `try_new()`, remove the two `VecDeque::with_capacity` calls for freq deques and add:\n\n```rust\nresident_map: HashMap::with_capacity(capacity),\n```\n\nRemove the `const SCAN_CAP: usize = 16;` line and the `scanned` counter logic from `evict_main()` (see §5 below).\n\n### 3. Rewrite `get()` with a single O(1) lookup\n\n```rust\npub fn get(&mut self, key: &K) -> Option<&V> {\n    if let Some((val, freq)) = self.resident_map.get_mut(key) {\n        *freq = freq.saturating_add(1).min(MAX_FREQ);\n        return Some(val);  // &mut V coerces to &V; lifetime bound to &mut self\n    }\n    None\n}\n```\n\nNo `position()` calls remain. A single `HashMap::get_mut` covers hit detection, frequency increment, and value retrieval in one lookup.\n\n### 4. Rewrite `put()` with O(1) update and admission\n\n```rust\npub fn put(&mut self, key: K, value: V) {\n    // O(1) update path: key already resident.\n    if let Some((v, _)) = self.resident_map.get_mut(&key) {\n        *v = value;\n        return;\n    }\n\n    // Ghost hit: O(1) via ghost_set; lazy deletion preserved.\n    if self.ghost_set.contains(&key) {\n        self.ghost_set.remove(&key);\n        if self.main.len() >= self.main_size {\n            self.evict_main();\n        }\n        self.resident_map.insert(key.clone(), (value, 0));\n        self.main.push_front(Entry::new(key, value));  // value arg ignored by Entry\n        return;\n    }\n\n    // Normal admission into small.\n    if self.small.len() >= self.small_size {\n        self.evict_small();\n    }\n    self.resident_map.insert(key.clone(), (value, 0));\n    self.small.push_front(Entry::new(key, value));\n}\n```\n\n### 5. Rewrite `evict_main()` — unbounded second-chance, no SCAN_CAP\n\nRemove `const SCAN_CAP` and the `scanned` counter entirely:\n\n```rust\nfn evict_main(&mut self) {\n    while let Some(tail) = self.main.pop_back() {\n        let freq = self.resident_map.get(&tail.key).map_or(0, |&(_, f)| f);\n        if freq > 0 {\n            if let Some((_, f)) = self.resident_map.get_mut(&tail.key) {\n                *f = freq - 1;\n            }\n            self.main.push_front(tail);\n        } else {\n            self.resident_map.remove(&tail.key);\n            break;\n        }\n    }\n}\n```\n\nThe unbounded scan terminates quickly on Zipfian workloads because cold entries accumulate naturally at the tail of main (confirmed in cycle 4).\n\n### 6. Rewrite `evict_small()` to use `resident_map`\n\n```rust\nfn evict_small(&mut self) {\n    if let Some(tail) = self.small.pop_back() {\n        let freq = self.resident_map.get(&tail.key).map_or(0, |&(_, f)| f);\n        if freq > 1 {\n            // Hot: promote to main; entry stays in resident_map.\n            if self.main.len() >= self.main_size {\n                self.evict_main();\n            }\n            self.main.push_front(tail);\n        } else {\n            // Cold: evict to ghost; remove from resident_map.\n            self.resident_map.remove(&tail.key);\n            while self.ghost.len() >= self.main_size {\n                if let Some(evicted_key) = self.ghost.pop_back() {\n                    if self.ghost_set.contains(&evicted_key) {\n                        self.ghost_set.remove(&evicted_key);\n                        break;\n                    }\n                }\n            }\n            let k = tail.key;\n            self.ghost.push_front(k.clone());\n            self.ghost_set.insert(k);\n        }\n    }\n}\n```\n\n### 7. Update the white-box unit test\n\nThe test `main_eviction_uses_frequency_second_chance` directly accesses `main_freq` (removed) and references `SCAN_CAP` (removed). Replace the direct field pushes with equivalent `resident_map` insertions:\n\n```rust\ncache.main.push_back(Entry::new(1, 10)); cache.resident_map.insert(1u64, (10u64, 0u8));\ncache.main.push_back(Entry::new(2, 20)); cache.resident_map.insert(2u64, (20u64, 2u8));\ncache.main.push_back(Entry::new(3, 30)); cache.resident_map.insert(3u64, (30u64, 0u8));\ncache.main.push_back(Entry::new(4, 40)); cache.resident_map.insert(4u64, (40u64, 1u8));\ncache.small.push_front(Entry::new(5, 50)); cache.resident_map.insert(5u64, (50u64, 2u8));\n```\n\nRemove the comment about `SCAN_CAP` from that test. All other tests use only the public API and require no changes.\n"
}
```

### 22:21:59 thinking

Let me verify the three files look correct by checking that they have the correct structure:
1. Claim (first non-empty non-header line after optional `#` header)
2. `## Parent` section
3. `## Changes` section

Let me also verify the filenames: H01.md, H02.md, H03.md — all zero-padded width 2 ✓

Let me do a quick sanity check on the files.

### 22:21:59 chat

Let me do a quick sanity check on the three files to confirm their structure is correct:

### 22:22:01 tool: Bash

```json
{
  "command": "for f in H01.md H02.md H03.md; do\n  echo \"=== $f ===\"\n  head -6 \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\hypotheses\\$f\"\n  echo \"---\"\n  grep \"^## Parent\" -A1 \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\hypotheses\\$f\"\n  echo \"\"\ndone",
  "description": "Verify structure of all three hypothesis files"
}
```

### 22:22:05 tool: Grep

```json
{
  "pattern": "^## Parent",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\hypotheses",
  "output_mode": "content",
  "context": 1
}
```

### 22:22:09 tool: Grep

```json
{
  "pattern": "^## Parent$|^cycle/|^baseline$",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\hypotheses",
  "output_mode": "content",
  "context": 1
}
```

### 22:22:24 chat

All three files are correctly structured. Here is a summary of what was written for cycle 0005:

---

## Cycle 0005 — 3 Hypotheses Written

All three hypotheses target the `position()` linear scan in `get()` and `put()` — the dominant latency bottleneck unanimously identified by cycle 4's two independent variants. Each pursues a distinct strategy and cites a cross-cycle winner as its parent.

### H01 — Full O(1) refactor via two side HashMaps
**Parent**: `cycle/0004/v002` (best incumbent, `small_size = capacity/40`)  
**Claim**: Replace `small_freq`/`main_freq` parallel deques and all `position()` calls with a `data: HashMap<K,V>` for O(1) value retrieval and a `freq_map: HashMap<K,u8>` for O(1) frequency access. Redefine `Entry<K,V>` to hold only the key (drops size from 16→8 bytes). Every `position()` call in `get()` and `put()` is eliminated; all paths become O(1).

### H02 — Minimal resident_set guard (miss-path only)
**Parent**: `cycle/0004/v002`  
**Claim**: Add a `resident_set: HashSet<K>` that enables an O(1) early exit on cache misses in both `get()` and `put()`. The hit path (`~73%` of accesses) is unchanged; only the miss path is fixed. Tests whether miss-path scan cost is a significant fraction of the per-op-time bottleneck — or whether the hit-path scan is the real culprit.

### H03 — Single merged HashMap (value + frequency in one lookup)
**Parent**: `cycle/0004/v001` (incumbent with `small_size = capacity/20`, bounded scan removed)  
**Claim**: Replace `small_freq`/`main_freq` and all `position()` calls with a single `resident_map: HashMap<K, (V, u8)>`. One hashmap lookup covers hit detection, value retrieval, and frequency update — half the hashmap round-trips of H01's two-map approach, potentially better cache locality. Also reverts the falsified bounded scan cap from v001 and holds `small_size = capacity/20`.


---

- finished: 2026-07-19T22:22:24+00:00
- duration: 515.09s
- status: ok
- input_tokens: 17
- output_tokens: 31913
- cache_read_tokens: 699898
- cache_write_tokens: 57645
- cost_usd: 0.906443
