# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0006\hypotheses
- allow_edits: True
- started: 2026-07-19T22:36:30+00:00

## timeline

### 22:36:34 thinking

Let me read all the necessary context files to understand the current state of the research and propose 3 distinct hypotheses for cycle 0006.

### 22:36:35 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\cycle-insight.md"
}
```

### 22:36:35 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\cycle-winners.json"
}
```

### 22:36:36 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\leaderboard.md"
}
```

### 22:36:36 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cross-cycle-insights.md"
}
```

### 22:36:39 thinking

Now let me read the cycle 0005 variants to understand what they implemented:

### 22:36:41 tool: Glob

```json
{
  "pattern": "v*/attempts/*/overlay/**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\variants"
}
```

### 22:36:41 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles-ledger.jsonl"
}
```

### 22:36:44 thinking

Now let me read the cycle 0005 variant overlays to understand exactly what v001, v002, and v003 implemented:

### 22:36:45 tool: Glob

```json
{
  "pattern": "v*/attempts/**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\variants"
}
```

### 22:36:49 tool: Bash

```json
{
  "command": "find \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\variants\" -type f 2>/dev/null || ls \"C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\variants\" 2>/dev/null",
  "description": "List cycle 0005 variant files"
}
```

### 22:36:52 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\variants"
}
```

### 22:36:55 thinking

Now let me read the overlay files for all three variants in cycle 0005 to understand their implementations:

### 22:36:56 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\variants\\v001\\attempts\\01\\overlay\\src\\s3_fifo\\mod.rs"
}
```

### 22:36:56 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\variants\\v003\\attempts\\01\\overlay\\src\\s3_fifo\\mod.rs"
}
```

### 22:36:57 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\variants\\v002\\attempts\\01\\overlay\\src\\s3_fifo\\mod.rs"
}
```

### 22:37:00 thinking

Now let me also read the baseline and the variant insights:

### 22:37:01 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\variants\\v001\\insight.md"
}
```

### 22:37:02 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\variants\\v002\\insight.md"
}
```

### 22:37:03 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0005\\variants\\v003\\insight.md"
}
```

### 22:37:03 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\baseline\\src\\s3_fifo\\mod.rs"
}
```

### 22:37:07 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\baseline"
}
```

### 22:37:09 tool: Glob

```json
{
  "pattern": "src/**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo"
}
```

### 22:37:12 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo"
}
```

### 22:37:15 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\src\\s3_fifo\\mod.rs"
}
```

### 22:37:16 tool: Read

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\inputs\\subjects\\s3_fifo\\CONTEXT.md"
}
```

### 22:40:04 thinking

Now I have a thorough understanding of the system. Let me synthesize the key insights:

**Current state (best incumbents):**
- cycle/0005/v001: hit-rate=0.7292, metadata-size=8, per-op-time=0
  - Uses dual HashMaps (data + freq_map), 1/40 small queue fraction
  - Two separate hash probes per `get()`
- cycle/0005/v002: hit-rate=0.7292, metadata-size=16, per-op-time=0
  - Uses resident_set HashSet (miss guard) + parallel freq deques, 1/40 small queue fraction
  - Still has O(n) position() on hit path, just skips miss path
- cycle/0005/v003: hit-rate=0.7279, metadata-size=8, per-op-time=100
  - Uses unified resident_map<K,(V,u8)>, 1/20 small queue fraction
  - Single hash probe per get(), cleanest design

**Key open questions from cycle insights:**
1. Is sub-1/20 small-queue fraction viable again now that position() is eliminated?
2. What is the new dominant per-operation cost after position() is gone?
3. v001 uses two separate hash probes per get() (freq_map + data), while v003 consolidates into one—wondering if that tradeoff is worth the hit-rate loss at scale.

**Strategies to explore for cycle 0006:**

1. **Sub-1/20 small-queue fraction on v003's resident_map design**: Since position() overhead is gone, I want to test whether pushing the small queue fraction down to 1/40 on v003's single-map design can improve hit-rate without sacrificing per-op performance.

2. **Merging v001's dual HashMaps**: v001 pays the cost of two probes per get() with its separate data and freq_map, while v003 does it in one. I'm wondering if consolidating those maps in v001 could reduce per-op-time while keeping its hit-rate advantage.

3. **Ghost capacity reduction**: The ghost queue currently mirrors main_size, which grows as the small queue shrinks. Testing a smaller ghost (maybe half the main size) could cut memory overhead without tanking hit-rate.

4. **Frequency ceiling change**: MAX_FREQ=3 might be overkill — trying MAX_FREQ=1 (binary hot/cold distinction) could simplify the logic and reduce metadata overhead.

5. **Lazy ghost tombstone aging**: The ghost backlog accumulates under heavy ghost-hit workloads. Instead of the current capacity guard approach, I could drain stale entries more aggressively, though this gets complex.

Now I'm narrowing down to the three most promising and distinct hypotheses. H01 would merge v001's dual HashMaps into a single resident_map like v003 does, but keep v001's smaller queue ratio.

The key insight is that v001 does two hash probes per get() while v003 does one, which could matter at larger working sets even though per-op-time is already zero on the current Zipfian benchmark. But I need to check whether this actually shows improvement given that both versions have the same metadata size.

Looking at what's still unexplored: I could try pushing v003's small queue down to 1/40 instead of 1/20 to see if hit-rate improves, or I could tackle the eviction churn in the main deque by replacing the physical move-to-front with something like a clock hand or segmented hot/cold halves—though that would require significant restructuring.

On the ghost queue front, reducing its size from main_size to capacity/2 would save memory, but that overhead lives in the S3Fifo struct itself, not in the Entry metadata that the fitness dimension actually measures, so it wouldn't show up as an improvement there.

Looking at what metadata-size actually scores — it's the Entry size (8 bytes for Entry<u64, u64>), not the S3Fifo struct overhead. The current Entry struct just holds a key and a zero-sized PhantomData marker, so there's not much room to shrink it below 8 bytes.

That leaves me focusing on the remaining optimization opportunities: hit-rate could potentially improve with better queue sizing or ghost policies, per-op-time is already at p50=0 which seems optimal, and metadata-size is at its floor. I should explore whether there are any other angles for hit-rate improvement or if I've hit the practical limits here. the small-queue fraction could potentially go even smaller than 1/40 on Zipfian workloads since the position scan cost is eliminated. I should also reconsider whether ghost-hit promotion to the main queue front is optimal—reinserting at the tail would likely hurt performance. The frequency threshold question was already tested in v003 and ruled out, so I need to think more carefully about what other levers might improve hit-rate.

Per-operation timing is already bottomed out at 0 ns, so that's not a useful optimization target. The real question is whether hit-rate can exceed the current best of 0.7292—either by shrinking the small-queue further or by rethinking the ghost queue strategy.

I'm considering a few concrete hypotheses: combining v001's aggressive 1/40 small-queue sizing with v003's cleaner single resident_map design to reduce hash probes, though the benchmark might not show measurable gains at this scale. More promisingly, I could test shrinking the small-queue to 1/80 on top of v003's structural foundation, building on the cross-cycle insight that smaller queues warrant re-testing.

The 1/80 approach targets Zipfian workloads where most accesses hit a few hot keys—a tiny small-queue acts as an admission filter to keep one-hit-wonders out of the main queue, freeing up capacity for those hot keys and potentially boosting hit-rate, though there's a risk that an extremely small queue might hurt cold-miss handling. Alternatively, I could merge v001's 1/40 sizing with v003's resident_map structure, replacing v001's dual HashMap approach with the single resident_map to cut down on hash probes while keeping the proven queue ratio.

But at p50=0 that won't move the needle much on per-op-time. The ghost queue is a better angle—it's currently bounded at main_size but lazy deletion lets it grow transiently above that, creating a tombstone backlog of stale entries. If I track the live ghost count more carefully, I could prevent that accumulation.

Actually, that's more of a correctness issue than a fitness lever. Let me refocus on what actually moves the metrics: hit-rate improves with smaller small-queue sizes (1/80 or 1/100 instead of 1/40) or by expanding the ghost queue for better recall, while per-op-time is already pretty constrained. test if merging the dual HashMap design in v001 into a single resident_map could reduce per-get() from 2 probes to 1, though this won't actually change the Entry metadata size since both versions already use 8-byte key-only entries, and both are already at p50=0 for per-op-time anyway.

The real testable improvements seem to be hit-rate gains by shrinking the small queue further or expanding the ghost queue, eviction efficiency by potentially eliminating the second-chance loop through a separate cold-tail structure, or tuning the admission policy itself.

I'm settling on three concrete hypotheses to test: combining v001's dual HashMap approach with a 1/40 small queue to see if it balances all three performance axes, pushing v003's small queue down to 1/80 to test whether further shrinkage continues improving hit-rate on Zipfian workloads, and exploring whether admitting ghost-promoted keys at positions proportional to their frequency rather than at the front yields better results. Since the ghost queue already covers nearly all capacity when small_size is minimal, expanding it further wouldn't add much. Let me explore a different angle: frequency-aware ghost scoring that places ghost-promoted keys based on their frequency history, but that requires position-based insertion which brings back the O(n) overhead. Instead, I'm considering whether simply lowering the frequency cap from 3 to 2 in v003 might improve performance.

With MAX_FREQ=2, hot entries get fewer second-chance opportunities before eviction, which could reduce the iterations in the evict_main loop and improve per-op-time, though it risks hurting hit-rate if the workload needs more frequency credits. The cycle analysis suggests evict_main's second-chance loop is the last major structural bottleneck, so testing this trade-off would be informative—though per-op-time is already at 0 p50 for earlier versions, so the gains might be minimal.

Looking at what could actually move the fitness metrics: hit-rate is the most tractable lever through a smaller small-queue fraction, and v003 has room to improve per-op-time since it's at 100 ns p50 compared to v001's 0 ns. That's the real gap to investigate. Looking at the performance metrics more carefully, v003 shows 100 ns p50 for both get and put operations, while v001 has 0 ns p50 for get but 100 ns for put. The summary fitness value for v001 is 0, which suggests it's using the get p50 as the primary metric rather than the maximum across operations. I need to figure out exactly how these summary values are being calculated to understand why v001 is winning despite both versions having similar put latencies. be faster than two probes, yet v003 is slower. The difference might stem from the queue size affecting miss rates in the small queue, or possibly the benchmarking granularity is too coarse to distinguish performance at the sub-100ns level where these measurements are noisy.

What's puzzling is that v001 with two HashMap probes shows 0ns while v003 with one probe shows 100ns—this could be measurement noise, or v003's resident_map storing (V, u8) tuples might have worse cache locality than v001's separate freq_map with just u8 values, leading to more cache misses on the frequency lookup path. The value map in v001 uses 16 bytes per entry while v003's combined tuple uses 24 bytes, making v001 more cache-efficient for the hot working set on a Zipfian distribution. This explains why v001's two probes still beat v003's single probe — the smaller, denser frequency map wins on cache locality. I'm wondering if separating the frequency storage in v003 could close this gap.

So I'm settling on two concrete hypotheses to test: first, pushing v001 even further with a smaller admission queue fraction (1/80 instead of 1/40) to see if we can break above 73% hit-rate on Zipfian workloads, and second, redesigning v003 to split frequency tracking into its own compact map alongside the resident map, which would essentially recover v001's cache advantage while keeping the simpler single-probe lookup structure. All three are cross-cycle winners so any works. H02 as currently framed—splitting freq_map from resident_map on v003—is just testing queue fraction differences, which feels redundant since v001 already covers 1/40. Instead, I should reframe H02 to test whether doubling the ghost queue capacity in v003 improves hit-rate by keeping evicted keys in the ghost window longer, letting them get promoted back on re-request in Zipfian workloads.

The change is straightforward: increase the ghost queue initialization and the eviction guard to use 2× main_size instead of 1×. The memory overhead is minimal since the ghost queue only stores keys, not values—roughly 16KB more. If more evicted keys can be ghost-promoted, hit-rate should improve, making this a genuinely distinct hypothesis worth testing. Splitting the frequency counter into its own HashMap to reduce per-entry size and cache pressure — keeping the value in a separate data map and probing frequency first on get operations, combining v003's smaller queue fraction with v001's dual-HashMap approach to potentially lower get latency from 100ns to near-zero. The empirical data contradicts the assumption that fewer probes means faster performance — v001's two-probe approach with smaller per-entry data actually outperforms v003's single probe. H03 would test whether the cache-friendly freq_map structure is the real advantage, independent of probe count. Both H02 and H03 reference v003 as a baseline, which is valid for the hypothesis testing framework.

For H01, pushing the ghost queue ratio from 1/40 to 1/80 aligns with the cycle insights that suggested smaller ratios might now be viable after the position() elimination. Since v001 already uses 1/40 successfully, extending to 1/80 is a natural continuation of that experiment rather than a repeat.

H02 and H03 target different fitness dimensions—H02 expands temporal coverage through a larger ghost queue, while H03 optimizes per-operation time by splitting the resident map into separate data and frequency structures for better cache locality. These are sufficiently distinct approaches.

Now I need to check which gated files would require modifications for these hypotheses, starting with the core JNI implementation, build configuration, and test files.

### 22:40:04 tool: Glob

```json
{
  "pattern": "**",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0006"
}
```

### 22:40:15 thinking

I need to create the hypotheses directory and the hypothesis files. The directory `C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0006\hypotheses` doesn't exist yet (or may be newly created when I write to it).

Let me finalize my 3 hypotheses and write the files.

**H01** (cites cycle/0005/v001 — cross-cycle winner):
- Claim: Reducing the small-queue fraction from 1/40 to 1/80 on cycle/0005/v001's dual-HashMap design will further improve hit-rate on Zipfian workloads by shifting even more capacity to the main queue, now that position() scan overhead has been eliminated.
- Parent: cycle/0005/v001
- Changes: In `try_new()`, replace `capacity / 40` with `capacity / 80`.

**H02** (cites cycle/0005/v003 — cross-cycle winner):
- Claim: Expanding the ghost queue capacity to twice main_size in cycle/0005/v003's resident_map design will improve hit-rate by widening the temporal window for ghost-promotion, allowing more recently evicted keys to be recaptured when re-requested.
- Parent: cycle/0005/v003
- Changes: Add a `ghost_capacity` field = `2 * main_size`, use it in the ghost guard condition, and initialize the ghost deque with that capacity.

**H03** (cites cycle/0005/v003 — cross-cycle winner):
- Claim: Splitting cycle/0005/v003's unified resident_map into separate frequency and data maps will reduce get() latency by improving cache-line utilization on the frequency-update hot path, since the frequency probe no longer requires scanning the full tuple.
- Parent: cycle/0005/v003
- Changes: Replace the combined map with a frequency map and a data map, then update all access patterns across get(), put(), and eviction methods to query both maps independently.

Now I'm writing out these hypothesis files to the hypotheses directory.

### 22:40:20 chat

Now I have all the context I need. Let me synthesize the insights and write three distinct hypotheses.

**Key findings:**
- Cycle 0005 eliminated all O(n) `position()` scans — that bottleneck is now gone
- Best incumbents: v001 (1/40 small queue, dual HashMaps, per-op-time=0) and v003 (1/20 small queue, single resident_map, per-op-time=100 surprisingly)
- v001 achieves get p50=0ns despite two HashMap probes; v003 achieves get p50=100ns with one probe — the compact `freq_map<K,u8>` is more cache-line-friendly than `resident_map<K,(V,u8)>`
- Open questions: can we push small-queue below 1/40? Can ghost queue expansion improve hit-rate?

Three distinct strategies:
1. **H01** (cites v001): Push small-queue fraction from 1/40 → 1/80 — test if Zipfian workloads tolerate even smaller admission filtering now that scan cost is gone
2. **H02** (cites v003): Expand ghost queue to 2× main_size — widen the recapture window for evicted keys
3. **H03** (cites v003): Split `resident_map<K,(V,u8)>` back into `data<K,V>` + `freq_map<K,u8>` — test if smaller per-entry size in the frequency map cuts v003's get p50 from 100ns to 0ns

### 22:40:29 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0006\\hypotheses\\H01.md",
  "content": "# H01\n\nReducing the small-queue fraction from 1/40 to 1/80 on cycle/0005/v001's dual-HashMap design will further improve hit-rate on Zipfian workloads by shifting more capacity to the main queue, now that the position() scan overhead that caused the cycle/0004 crossover has been eliminated.\n\n## Parent\n\ncycle/0005/v001\n\n## Changes\n\nIn `try_new()`, replace the single occurrence of `capacity / 40` with `capacity / 80`.\nThe expression is:\n```\n(capacity / 40).max(1).min(capacity - 1)\n```\nChange it to:\n```\n(capacity / 80).max(1).min(capacity - 1)\n```\nAll other code — the dual-HashMap layout (`data: HashMap<K,V>` + `freq_map: HashMap<K,u8>`), the lazy ghost tombstoning, the `ghost_set` HashSet, the `evict_main()` unbounded second-chance loop, and the `evict_small()` `freq > 1` promotion threshold — must be preserved exactly as in the parent. Only the small-queue sizing constant changes.\n\n## Rationale\n\ncycle/0005/v001 proved that 1/40 is safe now that `position()` is gone: hit-rate rose to 0.7292 with no latency penalty. The cross-cycle insight explicitly asks whether 1/40-or-smaller remains viable. At 1/80 the small queue holds ≈1.25% of capacity and main holds ≈98.75%, giving the hot Zipfian working set even more residency. On Zipfian workloads one-hit-wonders are rare by definition, so the admission filter can be made very tight without materially worsening cold-miss handling. The claim is falsified if hit-rate regresses below 0.7292 or if latency increases.\n"
}
```

### 22:40:41 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0006\\hypotheses\\H02.md",
  "content": "# H02\n\nExpanding the ghost queue capacity from main_size to twice main_size in cycle/0005/v003's resident_map design will improve hit-rate by widening the temporal window in which evicted keys can be ghost-promoted back into the main queue.\n\n## Parent\n\ncycle/0005/v003\n\n## Changes\n\nAdd a `ghost_capacity: usize` field to `S3Fifo` and initialize it to `2 * main_size` in `try_new()`. Update every place that references the ghost bound:\n\n1. **Struct definition** — add `ghost_capacity: usize` alongside `small_size` and `main_size`.\n\n2. **`try_new()`** — compute `ghost_capacity = 2 * main_size` (after `main_size` is derived), initialize `ghost: VecDeque::with_capacity(ghost_capacity)`, and store `ghost_capacity` in the struct. `ghost_set` should also be initialized with `HashSet::with_capacity(ghost_capacity)`.\n\n3. **`evict_small()`** — in the ghost capacity guard loop, replace the guard condition\n   ```rust\n   while self.ghost.len() >= self.main_size {\n   ```\n   with\n   ```rust\n   while self.ghost.len() >= self.ghost_capacity {\n   ```\n   The body of the loop (lazy tombstone drain: pop until a live ghost_set entry is found, then remove from ghost_set and break) is unchanged.\n\nAll other code — resident_map layout, get()/put() logic, evict_main() second-chance loop, evict_small() `freq > 1` promotion threshold, 1/20 small-queue fraction — must be preserved exactly as in the parent. Only the ghost queue bound changes.\n\n## Rationale\n\nThe ghost queue is currently bounded at `main_size` ≈ 95% of capacity (with 1/20 small queue). On Zipfian workloads, evicted entries concentrate in the long tail of the frequency distribution: they leave main after their second-chance frequency drains to zero, but some may be re-requested within a window longer than `main_size` accesses. Doubling the ghost window to `2 * main_size` costs only 8 bytes per key-slot (ghost stores keys only, no values), adding roughly `main_size * 8` bytes of deque memory. The claim is falsified if hit-rate does not improve beyond 0.7292 or if the additional memory pressure causes latency regression.\n"
}
```

### 22:41:00 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\s3_fifo_2026-07-20_00-22-12-011\\cycles\\0006\\hypotheses\\H03.md",
  "content": "# H03\n\nSplitting cycle/0005/v003's unified resident_map<K,(V,u8)> into a compact freq_map<K,u8> and a separate data<K,V> map will reduce get() latency from 100 ns to 0 ns p50 by improving cache-line utilization on the hot frequency-update path, matching cycle/0005/v001's latency while retaining v003's 1/20 small-queue fraction.\n\n## Parent\n\ncycle/0005/v003\n\n## Changes\n\nReplace the single `resident_map: HashMap<K, (V, u8)>` field with two fields:\n- `data: HashMap<K, V>` — stores the value for every resident key\n- `freq_map: HashMap<K, u8>` — stores only the frequency byte for every resident key\n\nThis mirrors cycle/0005/v001's dual-HashMap layout but preserves v003's 1/20 small-queue fraction and all other v003 invariants.\n\n**`S3Fifo` struct**: remove `resident_map`, add `data: HashMap<K, V>` and `freq_map: HashMap<K, u8>`.\n\n**`try_new()`**: initialize both maps with `HashMap::with_capacity(capacity)`.\n\n**`get()`**: probe `freq_map` first (mutable, updates frequency), then `data` (immutable, returns value reference):\n```rust\npub fn get(&mut self, key: &K) -> Option<&V> {\n    if let Some(freq) = self.freq_map.get_mut(key) {\n        *freq = freq.saturating_add(1).min(MAX_FREQ);\n    } else {\n        return None;\n    }\n    self.data.get(key)\n}\n```\n\n**`put()`**: resident-update check uses `self.data.contains_key(&key)` (or `self.data.get_mut`); on update, replace `entry.0 = value` with `self.data.insert(key, value); return;`. Ghost-hit and normal admission paths insert into both `data` and `freq_map` (freq initialized to 0), then push a key-only `Entry` into the relevant deque (same Entry struct as the parent with `_phantom: PhantomData<V>`).\n\n**`evict_main()`**: read frequency via `self.freq_map.get(&tail.key).copied().unwrap_or(0)`, update via `self.freq_map.insert(tail.key.clone(), freq - 1)`, and on eviction call both `self.data.remove(&tail.key)` and `self.freq_map.remove(&tail.key)`.\n\n**`evict_small()`**: same pattern — read frequency from `freq_map`, on ghost-path call `self.data.remove` and `self.freq_map.remove`.\n\nThe `Entry<K, V>` struct (key-only + `PhantomData<V>`), the `ghost` deque, `ghost_set` HashSet, `ghost_set`-based lazy tombstoning, the `evict_main()` unbounded second-chance loop, the `evict_small()` `freq > 1` promotion threshold, and the 1/20 small-queue fraction must all be preserved exactly as in the parent.\n\n## Rationale\n\ncycle/0005/v001 (dual HashMaps, 1/40 queue) achieves get p50 = 0 ns while cycle/0005/v003 (single resident_map, 1/20 queue) achieves get p50 = 100 ns. The difference: for Key=u64, V=u64 the `resident_map` stores `(u64, u8)` tuples (≈16 bytes per slot), whereas `freq_map` in v001 stores only `u8` values (≈9 bytes per slot). Fewer bytes per slot means more entries per cache line, reducing TLB and L1/L2 pressure on the hot `get()` frequency-bump path — the most frequent operation. This hypothesis isolates the data-layout variable by holding queue fraction constant (1/20, as in v003), letting us confirm or refute the cache-line hypothesis independently of queue sizing. The claim is falsified if get p50 does not drop below 100 ns, or if hit-rate regresses below v003's 0.7279.\n"
}
```

### 22:41:11 chat

Three hypotheses written to `cycles/0006/hypotheses/`:

---

**H01 — `cycle/0005/v001` → 1/80 small-queue fraction**
*Targets: hit-rate.*
Push the small-queue from 1/40 → 1/80 on v001's proven dual-HashMap design. At 1/80, main queue holds ~98.75% of capacity. The cycle/0004 crossover was scan-driven; with `position()` gone, even tighter admission filtering should be safe on Zipfian workloads. Falsified if hit-rate falls below 0.7292.

---

**H02 — `cycle/0005/v003` → 2× ghost queue capacity**
*Targets: hit-rate.*
Double the ghost queue bound from `main_size` to `2 * main_size` on v003's resident_map design. Ghost stores keys only (8 bytes each), so the overhead is modest (~16 KB for a 2048-capacity cache). The wider temporal window lets keys evicted earlier in a hot burst still be ghost-promoted rather than cold-admitted. Falsified if hit-rate doesn't exceed 0.7279.

---

**H03 — `cycle/0005/v003` → split `resident_map` into `data` + `freq_map`**
*Targets: per-op-time.*
Separate the `(V, u8)` tuple in v003's `resident_map` back into a compact `freq_map<K,u8>` plus `data<K,V>`, keeping v003's 1/20 queue fraction. The smaller per-entry size in `freq_map` (~9 bytes vs ~16 bytes) should yield better L1/L2 utilization on the hot frequency-bump path — explaining why v001 (two probes into compact maps) runs at 0 ns p50 while v003 (one probe into a fatter map) runs at 100 ns. Falsified if get p50 remains at 100 ns or hit-rate falls below 0.7279.


---

- finished: 2026-07-19T22:41:11+00:00
- duration: 281.47s
- status: ok
- input_tokens: 17
- output_tokens: 15375
- cache_read_tokens: 622420
- cache_write_tokens: 49926
- cost_usd: 0.606178
