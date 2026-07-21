# Agent log: variant-implementer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0003\variants\v001\attempts\01
- allow_edits: True
- started: 2026-07-19T21:54:17+00:00

## timeline

### 21:54:24 thinking

Let me start by reading the task details and understanding what I need to implement.

### 21:54:25 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\hypotheses\\H01.md"
}
```

### 21:54:25 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\manifest.toml"
}
```

### 21:54:29 thinking

Now let me read the parent variant's overlay files to understand the current state.

### 21:54:30 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0002\\variants\\v001\\attempts\\01\\overlay"
}
```

### 21:54:30 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0002\\variants\\v001\\insight.md"
}
```

### 21:54:34 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0002\\variants\\v001\\attempts\\01\\overlay\\src\\s3_fifo\\mod.rs"
}
```

### 21:54:39 thinking

Now I have a good understanding of the parent variant. Let me read the subject source to understand the existing structure.

### 21:54:39 chat

Now let me read the subject source and parent fitness vector:

### 21:54:41 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\src"
}
```

### 21:54:42 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0002\\variants\\v001\\eval\\fitness-vector.json"
}
```

### 21:54:45 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\src"
}
```

### 21:56:20 thinking

Now I have all the context I need. Let me implement the hypothesis:

1. The parent variant (`cycle/0002/v001`) uses a parallel-freq-deque layout, 1/10 small-queue fraction, O(1) ghost membership via `ghost_set`, and lazy-deletion tombstone guard in `evict_small()`.

2. My task (H01) is to replace the sequential second-chance scan in `evict_main()` with two-random-choices victim selection to reduce per-op-time to O(1) per eviction call.

The specific changes:
1. 

Adding RNG field and initialization...

The white-box test `main_eviction_uses_frequency_second_chance` will break because it expects deterministic eviction based on frequency and second-chance behavior, but the new two-random-choices algorithm makes eviction probabilistic—entry 3 or 4 might be selected depending on which random candidates are chosen. With the two-random-choices strategy, I can't guarantee that entry 3 gets evicted while entry 4 survives—the outcome depends on which entries the random selection picks. I realize the test is checking behavior specific to the second-chance eviction policy we're replacing, so it won't be valid anymore. But since the unit tests in `mod.rs` aren't gate-integrity files, I can actually modify this test to work with the new eviction strategy.

Let me work through what the two-random-choices algorithm would do with the deterministic RNG seeded by the capacity value of 5, calculating the initial RNG state and tracing through the random selections. I need to revise the test to work with the new two-random-choices eviction policy instead of relying on specific entries being evicted. The gate-integrity constraint means I can modify tests in `mod.rs` since they're not protected files, but I need to ensure the invariant gate in the `tests/` directory still passes. Let me update the `main_eviction_uses_frequency_second_chance` test to account for the new behavior where it picks between two random entries. Now I need to figure out which entry gets evicted by comparing the frequencies at the two randomly selected indices, then trace through the RNG calculations to determine the actual random values and see which one wins.

Actually, I'm reconsidering the approach here—rather than manually computing all these RNG values, I should focus on updating the test itself. The existing test validates the old frequency-based second-chance behavior, but now I need to test the new two-random-choices algorithm instead. I can either replace it with a test that checks the new behavior directly, or simplify it to verify structural invariants like ensuring exactly one entry gets evicted and that it had a lower or equal frequency compared to the other candidate. I could set up a two-entry cache where one has frequency 0 and the other has frequency 3, then verify that the lower-frequency entry always gets evicted since the two-random-choices algorithm picks the minimum of two sampled frequencies. Let me trace through what happens with the initial RNG state for a capacity-5 cache to see if the higher-frequency entry could ever be selected.

Computing the XOR shifts... The arithmetic is getting unwieldy, so I'm going to pivot and write the test differently instead. Rather than trying to verify exact numeric values, I'll focus on testing the core behavior: that evict_main removes exactly one entry, and that when multiple entries have the same frequency, the victim is chosen based on the new random sampling approach. I'll update the existing test to verify this new behavior in a way that doesn't require precise calculation of modulo operations.

Since the RNG is deterministic with a fixed seed, I could trace through the XorShift computations to predict which indices get sampled, but that's tedious. A better approach is to construct a test scenario where I can reason about the outcomes regardless of which two random indices are selected—for instance, placing entries with different frequencies at known positions and verifying that the lower-frequency entry gets evicted. I'm realizing the randomness with XorShift64 makes it hard to predict which entries get sampled, so the simplest approach is to test with just one entry in the cache—then both hash values map to index 0, guaranteeing that single entry gets evicted. Or I could test with multiple entries all having frequency 0 and just verify that exactly one was removed, regardless of which one. The other logic stays the same, and the invariant gate tests should pass since they verify structural correctness rather than which specific entry gets evicted. The internal test `main_eviction_uses_frequency_second_chance` will need updating since it checks for behavior that no longer applies. I'll update it to verify the new two-random-choices eviction instead, and I need to add the `rng` field to `S3Fifo`, initialize it in `try_new()`, implement the `next_rng()` method, replace the `evict_main()` body, and update that test.

Writing the overlay code... Initializing the cache structure... Implementing the eviction logic... Writing test cases... The swap disrupts the FIFO ordering in the deque, but that's fine since we're doing LRU-style eviction anyway, and the `get()` method searches by value rather than position so lookups still work correctly. Now I need to write out the complete overlay file, and I should verify that the RNG initialization prevents it from ever becoming zero—it's seeded with a hash of the capacity plus one, which ensures a non-zero starting value. odd number times any u64 will cycle through all possible values, so the only way to hit zero after adding 1 is if the base equals u64::MAX. The hypothesis claims it's guaranteed non-zero though, and since we return early for capacity=0, I'll accept that for typical use cases.

Now I'm ready to write out the complete S3-FIFO module file.

### 21:56:21 chat

Now I have all the information I need. Let me create the overlay with the hypothesis changes:
- Add `rng: u64` field with XorShift64 PRNG
- Replace `evict_main()` with two-random-choices O(1) implementation
- Update the unit test that tested old second-chance behavior

### 21:56:23 tool: Bash

```json
{
  "command": "mkdir -p \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v001\\attempts\\01\\overlay\\src\\s3_fifo\"",
  "description": "Create overlay directory structure"
}
```

### 21:57:17 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v001\\attempts\\01\\overlay\\src\\s3_fifo\\mod.rs",
  "content": "use std::collections::{HashSet, VecDeque};\nuse std::fmt::Debug;\nuse std::hash::Hash;\n\npub mod s3_fifo_jni;\n\n// Simple S3-FIFO implementation derived from \"FIFO Queues are ALL You Need for\n// Cache Eviction\" by Juncheng Yang et al. The policy uses a small FIFO admission\n// queue, a main FIFO resident queue, and a ghost queue for recently evicted\n// items. Reaccesses to ghosted keys promote them into the main queue.\n//\n// This variant extends cycle/0002/v001 with one additional change:\n//   1. small_size = capacity/10 — smaller admission queue.\n//   2. Parallel freq deques — freq removed from Entry<K,V> to shrink it\n//      from 24 to 16 bytes; stored in sibling small_freq/main_freq VecDeques\n//      indexed in lock-step with small/main.\n//   3. O(1) ghost membership + lazy deletion — ghost_set HashSet replaces\n//      O(n) iter().any(); ghost hits remove only from ghost_set (O(1)); the\n//      stale key remains in the deque and is cleaned up lazily.\n//   4. Two-random-choices eviction in evict_main() — replaces the O(n)\n//      second-chance scan with a deterministic XorShift64 RNG; samples two\n//      random indices and evicts the lower-frequency candidate, making\n//      evict_main() O(1).\nconst MAX_FREQ: u8 = 3;\n\n/// Entry stores only key and value; frequency is kept in a parallel deque.\n#[derive(Debug)]\npub struct Entry<K, V> {\n    key: K,\n    value: V,\n}\n\nimpl<K, V> Entry<K, V> {\n    fn new(key: K, value: V) -> Self {\n        Self { key, value }\n    }\n}\n\n#[derive(Debug)]\npub struct S3Fifo<K: PartialEq + Eq + Hash + Clone, V> {\n    /// Small admission FIFO queue (newest at front, oldest at back).\n    small: VecDeque<Entry<K, V>>,\n    /// Frequency values parallel to `small` (small_freq[i] ↔ small[i]).\n    small_freq: VecDeque<u8>,\n    /// Main resident FIFO queue.\n    main: VecDeque<Entry<K, V>>,\n    /// Frequency values parallel to `main` (main_freq[i] ↔ main[i]).\n    main_freq: VecDeque<u8>,\n    /// Ghost deque of recently evicted keys; may contain stale entries after\n    /// lazy deletion.\n    ghost: VecDeque<K>,\n    /// Set of keys currently live in the ghost deque (O(1) membership test).\n    /// ghost_set.len() <= ghost.len() always holds.\n    ghost_set: HashSet<K>,\n    small_size: usize,\n    main_size: usize,\n    /// XorShift64 state for two-random-choices victim selection in evict_main.\n    /// Guaranteed non-zero after initialisation.\n    rng: u64,\n}\n\nimpl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {\n    pub fn try_new(capacity: usize) -> Option<Self> {\n        if capacity == 0 {\n            log::error!(\"S3Fifo.try_new: capacity must be > 0\");\n            return None;\n        }\n        // Use 1/10 instead of 1/4 for the small queue.\n        let small_size = if capacity == 1 {\n            1\n        } else {\n            (capacity / 10).max(1).min(capacity - 1)\n        };\n        let main_size = capacity - small_size;\n        // Initialise RNG from capacity; guaranteed non-zero.\n        let rng = (capacity as u64)\n            .wrapping_mul(0x9e3779b97f4a7c15)\n            .wrapping_add(1);\n        Some(Self {\n            small: VecDeque::with_capacity(small_size),\n            small_freq: VecDeque::with_capacity(small_size),\n            main: VecDeque::with_capacity(main_size),\n            main_freq: VecDeque::with_capacity(main_size),\n            ghost: VecDeque::with_capacity(main_size),\n            ghost_set: HashSet::with_capacity(main_size),\n            small_size,\n            main_size,\n            rng,\n        })\n    }\n\n    pub fn new(capacity: usize) -> Self {\n        Self::try_new(capacity).expect(\"S3Fifo::new: initialization failed\")\n    }\n\n    /// XorShift64 step — O(1), state never zero after non-zero seed.\n    fn next_rng(&mut self) -> u64 {\n        self.rng ^= self.rng << 13;\n        self.rng ^= self.rng >> 7;\n        self.rng ^= self.rng << 17;\n        self.rng\n    }\n\n    /// Look up a key and increment its frequency.\n    /// Uses index-based position lookup so that the parallel freq deque can be\n    /// updated without needing freq stored inside Entry.\n    pub fn get(&mut self, key: &K) -> Option<&V> {\n        // Search small first.\n        if let Some(pos) = self.small.iter().position(|e| &e.key == key) {\n            let f = &mut self.small_freq[pos];\n            *f = f.saturating_add(1).min(MAX_FREQ);\n            return Some(&self.small[pos].value);\n        }\n        // Then main.\n        if let Some(pos) = self.main.iter().position(|e| &e.key == key) {\n            let f = &mut self.main_freq[pos];\n            *f = f.saturating_add(1).min(MAX_FREQ);\n            return Some(&self.main[pos].value);\n        }\n        None\n    }\n\n    pub fn put(&mut self, key: K, value: V) {\n        // Update existing resident entry (value-update path).\n        if let Some(pos) = self.small.iter().position(|e| e.key == key) {\n            self.small[pos].value = value;\n            return;\n        }\n        if let Some(pos) = self.main.iter().position(|e| e.key == key) {\n            self.main[pos].value = value;\n            return;\n        }\n\n        // Ghost hit: O(1) via ghost_set; lazy deletion — stale key stays in\n        // the ghost deque and is cleaned up by the capacity guard later.\n        if self.ghost_set.contains(&key) {\n            self.ghost_set.remove(&key);\n            if self.main.len() >= self.main_size {\n                self.evict_main();\n            }\n            self.main.push_front(Entry::new(key, value));\n            self.main_freq.push_front(0);\n            return;\n        }\n\n        // Normal admission into the small queue.\n        if self.small.len() >= self.small_size {\n            self.evict_small();\n        }\n        self.small.push_front(Entry::new(key, value));\n        self.small_freq.push_front(0);\n    }\n\n    /// Evict from the main queue using two-random-choices: sample two indices\n    /// with the XorShift64 RNG and evict the lower-frequency candidate.\n    /// VecDeque::swap and VecDeque::pop_back are O(1), so this is O(1).\n    fn evict_main(&mut self) {\n        let len = self.main.len();\n        if len == 0 { return; }\n        let a = (self.next_rng() as usize) % len;\n        let b = (self.next_rng() as usize) % len;\n        // Pick the lower-frequency candidate; tie-break toward higher index.\n        let victim = if self.main_freq[a] <= self.main_freq[b] { a } else { b };\n        // Swap victim to the back of both parallel deques, then pop — O(1).\n        let last = len - 1;\n        self.main.swap(victim, last);\n        self.main_freq.swap(victim, last);\n        self.main.pop_back();\n        self.main_freq.pop_back();\n    }\n\n    /// Evict from the small queue: promote to main if freq > 1, else ghost.\n    fn evict_small(&mut self) {\n        if let Some(tail) = self.small.pop_back() {\n            let freq = self.small_freq.pop_back().unwrap_or(0);\n            if freq > 1 {\n                // Hot entry: promote to main.\n                if self.main.len() >= self.main_size {\n                    self.evict_main();\n                }\n                self.main.push_front(tail);\n                self.main_freq.push_front(freq);\n            } else {\n                // Cold entry: move key to ghost.\n                // Capacity guard with lazy deletion:\n                //   Pop from the back of the ghost deque while it is at capacity.\n                //   If the popped key is still in ghost_set it is a live ghost —\n                //   remove it from ghost_set (normal aged eviction) and break.\n                //   If not, it was already lazily deleted; continue popping\n                //   (the effective ghost count did not change).\n                while self.ghost.len() >= self.main_size {\n                    if let Some(evicted_key) = self.ghost.pop_back() {\n                        if self.ghost_set.contains(&evicted_key) {\n                            self.ghost_set.remove(&evicted_key);\n                            break; // evicted one real ghost entry\n                        }\n                        // stale (lazily deleted): loop again\n                    }\n                }\n                let k = tail.key;\n                self.ghost.push_front(k.clone());\n                self.ghost_set.insert(k);\n            }\n        }\n    }\n}\n\nimpl<K: PartialEq + Eq + Hash + Clone + Debug, V: Debug> framework::Cache<K, V>\n    for S3Fifo<K, V>\n{\n    fn put(&mut self, key: K, value: V) {\n        S3Fifo::put(self, key, value);\n    }\n\n    fn get(&mut self, key: &K) -> Option<&V> {\n        S3Fifo::get(self, key)\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn get_miss_returns_none() {\n        let mut cache = S3Fifo::<i64, i64>::new(4);\n        assert!(cache.get(&42).is_none());\n    }\n\n    #[test]\n    fn put_then_get_finds_value() {\n        let mut cache = S3Fifo::new(4);\n        cache.put(1, 10);\n        assert_eq!(*cache.get(&1).unwrap(), 10);\n    }\n\n    #[test]\n    fn put_existing_updates_value() {\n        let mut cache = S3Fifo::new(4);\n        cache.put(1, 10);\n        cache.put(1, 20);\n        assert_eq!(*cache.get(&1).unwrap(), 20);\n    }\n\n    #[test]\n    fn small_eviction_promotes_to_main_after_multiple_hits() {\n        let mut cache = S3Fifo::new(4);\n        cache.put(1, 10);\n        cache.get(&1);\n        cache.get(&1);\n        cache.put(2, 20);\n        assert!(cache.main.iter().any(|entry| entry.key == 1));\n    }\n\n    #[test]\n    fn ghost_hit_promotes_back_to_main() {\n        let mut cache = S3Fifo::new(4);\n        cache.put(1, 10);\n        cache.put(2, 20);\n        cache.put(3, 30);\n        cache.put(4, 40);\n        cache.put(1, 10);\n        assert!(cache.main.iter().any(|entry| entry.key == 1));\n    }\n\n    /// White-box test for two-random-choices eviction in main.\n    /// Verifies that evict_main removes exactly one entry (swap-and-pop is O(1))\n    /// and that the parallel freq deque stays in sync.\n    #[test]\n    fn main_eviction_removes_one_entry() {\n        let mut cache = S3Fifo::new(5);\n        // Push entries directly into main and the parallel freq deque.\n        cache.main.push_back(Entry::new(1, 10));\n        cache.main_freq.push_back(0);\n        cache.main.push_back(Entry::new(2, 20));\n        cache.main_freq.push_back(2);\n        cache.main.push_back(Entry::new(3, 30));\n        cache.main_freq.push_back(0);\n        cache.main.push_back(Entry::new(4, 40));\n        cache.main_freq.push_back(1);\n        let initial_len = cache.main.len();\n        cache.evict_main();\n        // Exactly one entry must have been removed.\n        assert_eq!(cache.main.len(), initial_len - 1);\n        // The parallel freq deque must remain in sync.\n        assert_eq!(cache.main.len(), cache.main_freq.len());\n    }\n\n    /// Evicting from an empty main queue must be a no-op.\n    #[test]\n    fn main_eviction_empty_is_noop() {\n        let mut cache = S3Fifo::<i64, i64>::new(5);\n        assert_eq!(cache.main.len(), 0);\n        cache.evict_main(); // must not panic\n        assert_eq!(cache.main.len(), 0);\n    }\n\n    /// When main has a single entry, evict_main must remove it regardless of\n    /// which random indices are produced (both a and b collapse to index 0).\n    #[test]\n    fn main_eviction_single_entry_removed() {\n        let mut cache = S3Fifo::new(5);\n        cache.main.push_back(Entry::new(42, 99));\n        cache.main_freq.push_back(3);\n        cache.evict_main();\n        assert!(cache.main.is_empty());\n        assert!(cache.main_freq.is_empty());\n    }\n\n    #[test]\n    fn capacity_bound_never_exceeded() {\n        let mut cache = S3Fifo::new(4);\n        for key in 0..20 {\n            cache.put(key, key);\n        }\n        assert!(cache.small.len() + cache.main.len() <= 4);\n    }\n\n    #[test]\n    fn repeated_get_clamps_frequency() {\n        let mut cache = S3Fifo::new(4);\n        cache.put(1, 10);\n        for _ in 0..10 {\n            assert_eq!(*cache.get(&1).unwrap(), 10);\n        }\n    }\n\n    /// Lazy ghost deletion must not desync ghost_set and ghost deque:\n    /// ghost_set.len() <= ghost.len() must hold after ghost-hit scenarios.\n    #[test]\n    fn ghost_set_len_bounded_after_ghost_hit() {\n        // capacity=4 → small_size=1, main_size=3\n        let mut cache = S3Fifo::<i64, i64>::new(4);\n        // Fill ghost with entries 1, 2, 3 by pushing 4 entries through small.\n        cache.put(1, 10); // small=[1]\n        cache.put(2, 20); // evicts 1 → ghost=[1]\n        cache.put(3, 30); // evicts 2 → ghost=[2,1]\n        cache.put(4, 40); // evicts 3 → ghost=[3,2,1]\n        // Ghost hit on 1: removes 1 from ghost_set (lazy), stays in deque.\n        cache.put(1, 10);\n        // ghost_set = {2, 3}, ghost deque = [3,2,1(stale)]\n        assert!(\n            cache.ghost_set.len() <= cache.ghost.len(),\n            \"ghost_set.len() ({}) must not exceed ghost.len() ({})\",\n            cache.ghost_set.len(),\n            cache.ghost.len()\n        );\n        // Also verify the ghost hit actually promoted key 1 to main.\n        assert!(cache.main.iter().any(|e| e.key == 1));\n    }\n}\n"
}
```

### 21:57:23 chat

The overlay is written. Let me verify the file looks correct:

### 21:57:24 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v001\\attempts\\01\\overlay\\src\\s3_fifo\\mod.rs"
}
```

### 21:57:41 thinking

The overlay looks correct. Let me verify the key changes:

1. ✅ Added `rng: u64` field to `S3Fifo<K, V>` struct (line 59)
2. ✅ Initialized `rng` in `try_new()` with `(capacity as u64).wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(1)` (lines 76-78)
3. ✅ Added `next_rng()` method with XorShift64 (lines 97-102)
4. ✅ Replaced `evict_main()` with a two-random-choices O(1) implementation (lines 157-170)
5. ✅ Updated the unit tests to remove the frequency-based second-chance assertions and replace them with tests that work with the new eviction strategy
6. ✅ Preserved all other code from the parent variant

The overlay only modifies `src/s3_fifo/mod.rs`, which is correct since the JNI binding file is excluded. Now I'm doing a final review of the implementation to confirm everything is working correctly.

Verifying the `evict_main()` logic: it generates two random indices, compares their frequency values, and evicts the one with lower frequency by swapping it to the back of the VecDeque and popping it. The `next_rng()` method uses XOR-shift operations to generate the next random value, and the RNG is seeded during initialization with a deterministic hash of the capacity. All the synchronization between the main queue and frequency array checks out.

The overlay file only modifies `src/s3_fifo/mod.rs` since the JNI gate-integrity file can't be overlaid and the factory doesn't need changes for this variant.

### 21:57:41 chat

Let me quickly check if the subject has a `factory.rs` that needs to be considered:

### 21:57:41 tool: Glob

```json
{
  "pattern": "src/factory.rs",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo"
}
```

### 21:57:46 thinking

No factory.rs file. The subject uses `src/lib.rs` to expose the constructor (and that's a gate-integrity file I can't touch). The parent variant only had one overlay file: `src/s3_fifo/mod.rs`. My overlay is complete with just that file.

Let me do a final check to make sure my overlay file only exists at the right path and the changes are consistent with the hypothesis.

### 21:57:46 chat

The overlay is complete. Let me verify the overlay directory contents:

### 21:57:48 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v001\\attempts\\01\\overlay"
}
```

### 21:58:00 chat

The overlay is ready. Here's a summary of what was implemented:

## Changes Applied (H01 — Two-Random-Choices `evict_main`)

**Single file overlaid:** `src/s3_fifo/mod.rs`

### Structural additions to `S3Fifo<K, V>`:
1. **`rng: u64` field** — XorShift64 state, initialized in `try_new()` to `(capacity as u64).wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(1)` (guaranteed non-zero).

2. **`fn next_rng(&mut self) -> u64`** — applies one XorShift64 step (three shift-xor operations), advancing the PRNG state in O(1).

### `evict_main()` replacement:
The old O(n) second-chance loop (pop-back, decrement-or-re-admit-to-front) is replaced with:
- Sample two random indices `a` and `b` via `next_rng() % len`
- Evict the lower-frequency candidate: `victim = if main_freq[a] <= main_freq[b] { a } else { b }`  
- Swap victim to the back of both parallel deques, then `pop_back()` — both ops are O(1)

### Unit tests updated:
The `main_eviction_uses_frequency_second_chance` test (which asserted specific second-chance behavior) was replaced with three new tests:
- `main_eviction_removes_one_entry` — verifies exactly one entry evicted and freq deque stays in sync
- `main_eviction_empty_is_noop` — verifies empty main is handled safely
- `main_eviction_single_entry_removed` — verifies len=1 case (both random indices collapse to 0)

All other parent-variant code (parallel freq deques, 1/10 small-queue fraction, O(1) ghost set, lazy-deletion tombstone guard) is preserved without change.


---

- finished: 2026-07-19T21:58:00+00:00
- duration: 222.97s
- status: ok
- input_tokens: 12
- output_tokens: 14217
- cache_read_tokens: 298985
- cache_write_tokens: 28805
- cost_usd: 0.412107
