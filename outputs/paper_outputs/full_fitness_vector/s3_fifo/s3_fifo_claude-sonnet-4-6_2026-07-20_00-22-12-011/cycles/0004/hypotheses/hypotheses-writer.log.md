# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0004\hypotheses
- allow_edits: True
- started: 2026-07-19T22:02:42+00:00

## timeline

### 22:02:47 thinking

Let me read all the context files needed to write the hypotheses for cycle 0004. I'll start by reading the cross-cycle history files and the baseline, then read the cycle 0003 variants.

### 22:02:48 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\leaderboard.md"
}
```

### 22:02:48 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles-ledger.jsonl"
}
```

### 22:02:48 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cross-cycle-insights.md"
}
```

### 22:02:51 thinking

Now let me read the cycle 0003 variants in detail, along with the baseline.

### 22:02:53 tool: Glob

```json
{
  "pattern": "v*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants"
}
```

### 22:02:54 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\cycle-winners.json"
}
```

### 22:02:54 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\cycle-insight.md"
}
```

### 22:03:00 thinking

Now let me read the variant overlays from cycle 0003 to understand the current code.

### 22:03:00 chat

Now let me read the cycle 0003 variant overlays and insights:

### 22:03:00 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v001\\attempts\\01\\overlay"
}
```

### 22:03:00 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v002\\attempts\\01\\overlay"
}
```

### 22:03:01 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v003\\attempts\\01\\overlay"
}
```

### 22:03:04 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v002\\attempts\\01\\overlay\\src\\s3_fifo\\mod.rs"
}
```

### 22:03:05 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v001\\attempts\\01\\overlay\\src\\s3_fifo\\mod.rs"
}
```

### 22:03:05 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v003\\attempts\\01\\overlay\\src\\s3_fifo\\mod.rs"
}
```

### 22:03:09 thinking

Now let me read the variant insights for cycle 0003:

### 22:03:09 chat

Now let me read the variant insights:

### 22:03:11 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v001\\eval\\fitness-vector.json"
}
```

### 22:03:11 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v002\\eval\\fitness-vector.json"
}
```

### 22:03:12 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0003\\variants\\v003\\eval\\fitness-vector.json"
}
```

### 22:03:15 thinking

Now let me read the baseline source code:

### 22:03:15 chat

Now let me read the baseline:

### 22:03:15 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\baseline"
}
```

### 22:03:19 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\baseline\\insight.md"
}
```

### 22:03:19 tool: Glob

```json
{
  "pattern": "**/*.rs",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\baseline"
}
```

### 22:05:56 thinking

The baseline doesn't have Rust source files directly in it (they're in the inputs directory for subjects). Let me look at what files are available for the baseline more carefully.

Now I have a clear picture of the codebase and the evolutionary history. Let me now plan the 3 hypotheses for cycle 0004.

## Summary of current state

**Best winner**: `cycle/0003/v002` (hit-rate=0.7279, metadata-size=16, per-op-time=100)
- Uses: small_size = capacity/20 (1/20 fraction), parallel freq deques, O(1) ghost with lazy deletion, standard second-chance `evict_main()`
- The shrinking small-queue trend from 1/4 down to 1/20 has consistently improved performance, so 1/40 might push further, though there's likely a floor. The main bottleneck is still the O(n) second-chance scan in `evict_main()` with its pop_back/push_front churn. The two-random-choices approach from v001 cut latency but tanked hit-rate, so the real opportunity is capping the second-chance scan to just 8-16 candidates instead of scanning the whole queue.

I also notice the get path has its own O(n) scan problem when looking up positions in both queues, which could be worth addressing separately.

For the next round, I need to build hypotheses that cite a proven cross-cycle winner. The mandatory one is a bounded second-chance scan in `evict_main()` — building on `cycle/0003/v002` as the parent, I'd cap the scan at 16 iterations so it only evaluates a fixed window of candidates before evicting the one with the lowest frequency observed in that window. This directly tackles the open question from the last two cycles about whether tighter bounds help or hurt.

For a second hypothesis, I could push the small-queue shrinkage further — changing from `capacity / 20` to `capacity / 40` to give the main queue even more room. The trend has held steady for three cycles, so testing if it continues makes sense, though the real goal here is hit-rate improvement rather than latency. H03 would combine both approaches but might be redundant. Let me step back and consider something fundamentally different—what if ghost entries that get promoted back to the main queue retained their original frequency instead of starting at zero? This could preserve access patterns better than resetting them.

The catch is that storing frequency alongside ghost keys would bloat the metadata, so that's not practical. Another angle: when a key in the small queue gets a hit, I could move it to the front to extend its residency time, similar to a move-to-front heuristic, though that adds churn. Or maybe I should explore a segmented clock or frequency-bucketed promotion strategy instead.

Looking at the actual levers available in the code, the ghost deque capacity is one knob—it's currently sized to `main_size`, but I could experiment with shrinking or expanding it. A larger ghost means more opportunities to confirm demand on re-requests, but it costs memory per tracked key.

The frequency cap itself is another lever. Raising MAX_FREQ from 3 to 4 or 5 would let popular entries survive longer in the eviction scan, potentially improving hit-rate, though it'd slow down the second-chance pass. Lowering it would speed things up but might hurt performance on the most frequently accessed items.

I could also add admission filtering—maybe a Bloom filter or frequency counter to reject truly cold keys before they even enter the small queue, though that adds complexity and metadata overhead.

There's also the possibility of using a two-level frequency scheme in main, where instead of decrementing by 1 during second-chance, I decrement by a larger step like dividing by 2 or subtracting 2. This accelerates frequency decay and lets the scan terminate faster, trading some hit-rate for better per-operation time.

Looking at the most promising approaches: the bounded second-chance scan caps iterations at 16 pops and tracks the minimum-frequency entry to evict if we hit the limit—this feels like the right balance between the O(n) worst case and pure O(1). I should also continue shrinking the small queue from 1/20 to 1/40 of capacity.

For the frequency decay angle, I'm considering decrementing by 2 instead of 1 when re-admitting entries to the front. This shortens second-chance cycles without changing the worst-case behavior when everything has frequency 1. The tradeoff is that entries at frequency 2 or higher get fewer second chances, but they cycle through faster overall. step=1. This effectively compresses the frequency scale. The real goal here is improving per-operation time for evict_main() while accepting some hit-rate trade-off on high-frequency entries.

I'm realizing H01 and H03 are actually distinct enough—one bounds the scan iteration count directly, the other speeds up individual iterations by reducing how long entries survive. But there's another angle worth exploring: optimizing the get() path itself. Since the cache is heavily skewed toward main entries (about 95%), I could search main before small instead of the current order, which would reduce average search time.

Though actually, that's probably not worth it—small is only around 100 entries for a capacity of 2048, so the linear scan through it is negligible. I need something more impactful for H03.

What if I focus on the ghost deque itself? Right now tombstones pile up until the capacity guard in `evict_small()` runs and has to skip through them. Under heavy ghost-hit workloads, the deque can get bloated with stale entries while the actual live set is much smaller. I could add periodic cleanup—say, every 64 puts—to proactively drain dead tombstones from the back of the deque. This would make the capacity guard faster when it does run and reduce peak memory usage. Though I'm not sure if the benchmark would even show this difference.

Actually, let me step back. A cleaner hypothesis that hasn't been tested yet is increasing MAX_FREQ from 3 to 4, which would let the hottest entries survive longer in main before eviction. That trades off hit-rate improvement against longer per-operation time in `evict_main()`. Let me look more carefully at the cycle 3 v002 code to understand the get path better—it's doing O(n) scans through both small and main.

The put path also has those same O(n) scans when checking for existing entries, plus the ghost_set lookup. For new inserts (the common case), we're doing two full scans just to verify the key isn't already cached. I could optimize this by maintaining separate HashSets to track which keys live in small and main, eliminating those linear scans.

The tradeoff is adding constant per-instance overhead for the HashSets themselves, but gaining O(1) lookups for the "is this key resident?" check. The hot path where a key already exists still needs O(n) to find its position for the value update, but the cold path for new admissions becomes much faster. I could even go further and use a HashMap to store positions directly, avoiding the scan entirely. The resident_set would grow with capacity just like the other frequency deques, so that's not a new concern. Let me focus on three concrete hypotheses to test: the first caps the eviction scan at 16 iterations and evicts the lowest-frequency entry found, targeting per-operation latency improvements.

The second shrinks the small queue further from 1/20 to 1/40 of capacity, continuing the hit-rate gains I've been seeing. The third accelerates frequency decay by subtracting 2 instead of 1 during re-admission, so lower-frequency entries get evicted sooner while still surviving at least one cycle. Actually, I need to trace through this more carefully. With `freq - 1` in the code and a step of 2, an entry with freq=3 gets re-admitted at freq=1, then at freq=0, then evicted—that's three total encounters before eviction. But I'm second-guessing whether freq=0 actually triggers re-admission or if the `if freq > 0` check prevents that. saturating_sub with step=2 means freq=1 drops straight to 0, so a freq=1 entry gets evicted in 2 passes total. Let me trace through the actual evict_main code to see how frequencies are being decremented.

The evict_main function pops entries from the back of the main queue and checks their frequency. If freq > 0, it re-admits the entry at the front with freq decremented by 1, then continues the loop. If freq == 0, it breaks and evicts that entry. So the loop keeps cycling entries until it finds one with frequency 0 to actually remove.

With step=2 instead of step=1, entries reach freq=0 faster since they decrement by larger amounts each lap. An entry with initial freq=k needs k laps with step=1 but only ⌈k/2⌉ laps with step=2. For MAX_FREQ=3, this means entries need at most 3 laps versus 2 laps, which shortens the scan. The tradeoff is that hot entries lose some protection rounds.

This is a genuinely different approach from the bounded scan strategy and the queue sizing approach, so I'll treat it as a third hypothesis. I should verify that my hypotheses are actually distinct from each other and from prior work—H02 extends the sizing strategy while H03 changes the frequency decay rate, so they're clearly separate. I also need to confirm that at least one hypothesis cites a cross-cycle winner and that all strategies are sufficiently distinct.

Now I'm laying out the three hypothesis files, starting with H01, which proposes capping the second-chance eviction loop at a fixed iteration count with a minimum-frequency fallback to reduce latency spikes while maintaining hit-rate quality.

I need to work through the mechanics carefully—whether to track the minimum-frequency entry during the bounded scan and evict that if we hit the iteration limit, or to evict the last popped entry once we exceed 16 iterations. Writing the eviction logic...

When we hit the scan budget, we drop the entry we just popped without re-admitting it, which evicts it cleanly. This gives us O(min(n, SCAN_CAP+1)) behavior per eviction—much better than scanning the whole queue, and we still drain frequencies from the back entries before giving up. I'll go with this approach.

Now I'm considering reducing the small-queue fraction from 1/20 down to 1/40 to further improve hit rates on Zipfian traces by letting more entries stay in the main queue.

Modifying the frequency decay strategy...

With `saturating_sub(2)`, an entry with freq=1 drops to 0 and still passes the `if freq > 0` check, so it gets re-admitted with freq=0 before being evicted on the next scan. But I could tighten this by checking if the decremented frequency is still positive before re-admitting—that way only entries with freq≥3 survive, immediately evicting freq=1 and freq=2 entries. re-admit at 2 → re-admit at 1 → re-admit at 0 → evict. (4 passes)

So with step=2, freq=2 entries get 2 passes instead of 3, and freq=3 entries get 3 passes instead of 4. This speeds up eviction while keeping the second-chance mechanism intact. The trade-off is that freq=2 entries might not survive as long, but the per-operation time should improve. Both H01 and H03 are targeting the same per-op-time metric, just through different mechanisms.

Alternatively, I could pivot H03 toward hit-rate instead. The main queue holds about 95% of entries, so most hits happen there. I could add a HashSet to track which keys are in the main queue, letting me do an O(1) presence check before scanning through the queue itself.

This would speed up misses significantly—I'd skip the expensive O(n) scan through main entirely once I confirm the key isn't there. Hits in main would still require the O(n) scan to find the position, so that path doesn't improve. The tradeoff is maintaining the HashSet during insertions and evictions, which adds overhead. The miss path is actually faster than the hit path right now, so I need to reconsider the tradeoff. I'll stick with H03 as the step=2 decay mechanism in evict_main() since it's concrete and testable. Let me finalize the three hypotheses: H01 targets get latency with a bounded second-chance scan, H02 targets hit-rate by reducing the small queue to 1/40, and H03 targets per-operation time with the frequency decay step.

All three cite cycle/0003/v002 as the parent, which satisfies the cross-cycle winner requirement. The strategies are distinct—H01 modifies how many entries evict_main() checks, H02 changes the capacity allocation formula in try_new(), and H03 adjusts the frequency decrement amount in evict_main().

H01 and H03 both touch evict_main() but differ meaningfully: one adds a structural loop cap while the other tweaks a parameter, with different hit-rate implications. I'm ready to generate the hypothesis files in the output directory.

### 22:05:59 chat

Now I have all the context needed. Let me write the three hypotheses for cycle 0004.

**Summary of reasoning:**

- **Best incumbent**: `cycle/0003/v002` (hit-rate=0.7279, 1/20 small queue, parallel freq deques, O(1) ghost with lazy deletion, standard full O(n) second-chance `evict_main()`)
- **Primary open question** identified across 3 cycles: bounded second-chance scan in `evict_main()` — can a scan cap recover the latency gain from two-random-choices (cycle/0003/v001) while preserving hit-rate?
- **Secondary open question**: has the small-queue shrinkage trend (1/4→1/10→1/20) hit its floor yet?
- **Unexplored angle**: faster frequency decay rate (step=2 instead of step=1) in `evict_main()` shortens per-entry protection window, reducing scan length without eliminating second-chance semantics

### 22:06:14 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\hypotheses\\H01.md",
  "content": "# H01\n\nCapping `evict_main()`'s second-chance loop at 16 iterations before forcing eviction will reduce get p99 latency while preserving most of the hit-rate quality that full O(n) second-chance provides.\n\n## Parent\n\ncycle/0003/v002\n\n## Changes\n\nReplace the unbounded `while let Some(tail) = self.main.pop_back()` loop in `evict_main()` with a bounded variant that introduces a `SCAN_CAP: usize = 16` constant and a `scanned: usize` counter.\n\nEach iteration proceeds exactly as before: pop the tail entry and its freq. If `freq == 0`, evict it (drop the entry, break). If `freq > 0` **and** `scanned < SCAN_CAP`, decrement freq, push_front to re-admit, increment `scanned`, and continue. If `freq > 0` **and** `scanned >= SCAN_CAP`, the budget is exhausted — drop the entry (evict it without re-admission) and break.\n\n```rust\nfn evict_main(&mut self) {\n    const SCAN_CAP: usize = 16;\n    let mut scanned = 0;\n    while let Some(tail) = self.main.pop_back() {\n        let freq = self.main_freq.pop_back().unwrap_or(0);\n        if freq == 0 || scanned >= SCAN_CAP {\n            // Evict: either zero-freq natural victim or scan budget exhausted.\n            break;\n        }\n        // Re-admit with second-chance decrement.\n        self.main.push_front(tail);\n        self.main_freq.push_front(freq - 1);\n        scanned += 1;\n    }\n}\n```\n\nAll other functions (`evict_small`, `put`, `get`, `try_new`) remain identical to `cycle/0003/v002`. The `entry` and `S3Fifo` struct layouts are unchanged. The existing unit test `main_eviction_uses_frequency_second_chance` remains valid; update its assertion comments to note the cap exists but the test's queue (4 entries, mixed freq) will still find a zero-freq entry before hitting the cap.\n\n**Invariant to preserve**: `evict_main()` must always remove exactly one entry from `self.main` (and one from `self.main_freq`) per call. The bounded loop preserves this: either the `freq == 0` branch breaks (removing that entry) or the `scanned >= SCAN_CAP` branch breaks (removing the entry popped in that iteration). An early-exit when `main.is_empty()` is not needed here since `evict_main` is only called when `main.len() >= main_size > 0`.\n\n## Rationale\n\ncycle/0003/v001 demonstrated that the deque-churn inside `evict_main()` (not scan length per se) dominates get latency, but pure two-random-choices sacrificed hit-rate materially. A bounded scan of 16 candidates limits worst-case churn to 16 `pop_back`/`push_front` pairs while still draining frequency from the back of the queue — preserving demand-confirmation quality for all entries that make it through the first 16 slots. The cap value 16 is chosen to be a small constant (cache-line friendly) large enough to encounter zero-freq entries on most eviction calls under Zipfian workloads where the majority of entries are cold.\n"
}
```

### 22:06:27 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\hypotheses\\H02.md",
  "content": "# H02\n\nReducing the small-queue fraction from 1/20 to 1/40 will further improve hit rate on Zipfian traces by expanding main-queue residency to approximately 97.5% of total capacity.\n\n## Parent\n\ncycle/0003/v002\n\n## Changes\n\nIn `try_new()`, change the small-queue sizing formula from `capacity / 20` to `capacity / 40`. The `.max(1).min(capacity - 1)` guards must remain in place to protect against edge cases at small capacities. No other code changes.\n\nBefore:\n```rust\nlet small_size = if capacity == 1 {\n    1\n} else {\n    (capacity / 20).max(1).min(capacity - 1)\n};\n```\n\nAfter:\n```rust\nlet small_size = if capacity == 1 {\n    1\n} else {\n    (capacity / 40).max(1).min(capacity - 1)\n};\n```\n\nAll other functions, struct fields, and constants remain identical to `cycle/0003/v002`.\n\nThe `S3Fifo<u64,u64>` control struct size may change slightly (the `ghost` and `ghost_set` fields are pre-allocated to `main_size`, which grows from `capacity * 19/20` to `capacity * 39/40`). The `Entry<u64,u64>` size remains 16 bytes.\n\nUpdate the inline comment above the formula from \"H02 change: use 1/20 instead of 1/10\" to \"H02 change: use 1/40 instead of 1/20\".\n\n## Rationale\n\nThe small-queue fraction has been halved in every prior cycle (1/4 → 1/10 → 1/20) and each reduction improved hit rate and get latency with no observed penalty on the Zipfian workload-a trace. The trend has not yet shown a floor. At 1/40, main receives approximately 39 out of every 40 capacity slots, further reducing the admission-filtering overhead. The risk is that an overly thin small queue may fail to absorb cold-start bursts, but the ghost channel's demand-confirmation mechanism compensates: keys that re-surface quickly are still promoted to main regardless of small queue depth.\n"
}
```

### 22:06:42 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0004\\hypotheses\\H03.md",
  "content": "# H03\n\nDecrementing frequency by 2 instead of 1 in `evict_main()`'s second-chance re-admit path will shorten the scan length for moderately-warm entries and reduce get latency without the hit-rate cost of eliminating second-chance altogether.\n\n## Parent\n\ncycle/0003/v002\n\n## Changes\n\nIn `evict_main()`, change the re-admission frequency from `freq - 1` to `freq.saturating_sub(2)`. The `if freq > 0` guard remains unchanged, so any entry with freq > 0 still receives at least one second-chance re-admission.\n\nBefore:\n```rust\nfn evict_main(&mut self) {\n    while let Some(tail) = self.main.pop_back() {\n        let freq = self.main_freq.pop_back().unwrap_or(0);\n        if freq > 0 {\n            self.main.push_front(tail);\n            self.main_freq.push_front(freq - 1);\n        } else {\n            break;\n        }\n    }\n}\n```\n\nAfter:\n```rust\nfn evict_main(&mut self) {\n    while let Some(tail) = self.main.pop_back() {\n        let freq = self.main_freq.pop_back().unwrap_or(0);\n        if freq > 0 {\n            self.main.push_front(tail);\n            self.main_freq.push_front(freq.saturating_sub(2));\n        } else {\n            break;\n        }\n    }\n}\n```\n\nNo other changes. All struct fields, `try_new`, `put`, `get`, and `evict_small` are identical to `cycle/0003/v002`.\n\n**Effect on re-admission counts** (with `MAX_FREQ = 3`):\n\n| Initial freq | Step=1 passes to eviction | Step=2 passes to eviction |\n|---|---|---|\n| 1 | 2 (1→0→evict) | 2 (1→0→evict, same) |\n| 2 | 3 (2→1→0→evict) | 2 (2→0→evict) |\n| 3 | 4 (3→2→1→0→evict) | 3 (3→1→0→evict) |\n\nEntries at freq=1 behave identically to step=1. Entries at freq=2 lose one protection round. Entries at freq=3 lose one protection round. The second-chance policy is preserved — no entry is ever evicted at first encounter without at least one re-admission attempt — but the worst-case scan length for a fully-warm queue is cut from 4× capacity to 3× capacity.\n\nThe existing unit test `main_eviction_uses_frequency_second_chance` checks that an entry with freq=1 survives and an entry with freq=0 is evicted immediately. Both assertions remain valid under step=2. Update the test comment to note that freq=2 entries now behave like freq=1 entries under step=2.\n\n## Rationale\n\ncycle/0003/v001's two-random-choices result confirmed that reducing `evict_main()` scan length yields latency gains, but that pure O(1) random victim selection sacrifices frequency-drain protection, causing material hit-rate regression. Frequency step=2 is a middle path: the scan loop terminates faster (freq drains to zero in fewer passes) while preserving the invariant that hot entries (freq=3) still receive multiple protection rounds. Unlike the bounded-cap approach (H01), which caps iteration count, step=2 caps per-entry survival rounds — the two mechanisms are complementary and target different sources of scan cost.\n"
}
```

### 22:06:53 chat

All three hypotheses have been written to `C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0004\hypotheses\`.

---

## Summary

**H01** (`cycle/0003/v002` → bounded second-chance scan, targets **per-op-time**)
Adds a `SCAN_CAP = 16` iteration limit to `evict_main()`'s while loop. When the budget is exhausted, the next-popped entry is evicted unconditionally. This limits worst-case deque churn to 16 `pop_back`/`push_front` pairs — addressing the primary open question from cycles 2 and 3 — while preserving frequency-drain quality for queues that contain zero-freq entries within the first 16 back-slots (the common case on Zipfian workloads).

**H02** (`cycle/0003/v002` → small-queue fraction 1/40, targets **hit-rate**)
Continues the three-cycle monotone shrinkage trend (1/4 → 1/10 → 1/20 → 1/40). The only change is the sizing divisor in `try_new()`. Main queue grows to ~97.5% of capacity, further reducing admission-filtering overhead. Tests whether the trend has a floor.

**H03** (`cycle/0003/v002` → frequency decay step=2, targets **per-op-time**)
Changes `freq - 1` to `freq.saturating_sub(2)` in the re-admit path of `evict_main()`. Entries at freq=2 now drain to zero in one pass instead of two; entries at freq=3 in two passes instead of three. Second-chance semantics are preserved (no entry evicted on first encounter). This shortens the expected scan length with a modest hit-rate trade-off only for freq=2 entries, and is mechanistically distinct from H01's iteration-count cap.


---

- finished: 2026-07-19T22:06:53+00:00
- duration: 251.38s
- status: ok
- input_tokens: 12
- output_tokens: 14193
- cache_read_tokens: 373080
- cache_write_tokens: 40535
- cost_usd: 0.478435
