# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/hypotheses
- allow_edits: True
- started: 2026-10-28T19:30:27+00:00

## timeline

### 19:30:31 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262; cat leaderboard.md cross-cycle-insights.md cycles/0008/cycle-insight.md; tail -c 6000 cycles-ledger.jsonl; ls cycles/0008/variants; for v in cycles/0008/variants/*; do echo $v; cat $v/insight.md; ls -R $v/attempts | head -30; done"
}
```

### 19:30:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cross-cycle-insights.md"
}
```

### 19:30:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/leaderboard.md"
}
```

### 19:30:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/cycle-insight.md"
}
```

### 19:30:36 chat

Now the v001 overlay.

### 19:30:37 tool: Glob

```json
{
  "pattern": "cycles/0008/variants/v001/**/*",
  "path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262"
}
```

### 19:30:40 tool: Grep

```json
{
  "pattern": "fn ",
  "path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/variants/v001",
  "output_mode": "files_with_matches"
}
```

### 19:30:42 thinking



### 19:30:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/variants/v001/insight.md"
}
```

### 19:30:43 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/variants/v001/attempts/01/overlay"
}
```

### 19:30:45 thinking



### 19:30:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/variants/v001/attempts/01/overlay/src/dash/mod.rs"
}
```

### 19:30:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles-ledger.jsonl"
}
```

### 19:30:51 thinking



### 19:30:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/segment.rs"
}
```

### 19:30:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/bucket.rs"
}
```

### 19:30:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/mod.rs"
}
```

### 19:30:54 thinking



### 19:30:54 chat

I'm trying to find the cycle 0008 v001 source files, so the hypotheses can name specific functions.

### 19:30:55 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/hypotheses/H01.md"
}
```

### 19:30:56 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/cycle-winners.json"
}
```

### 19:30:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/variants/v001/attempts/01/overlay/src/dash/segments/dash_segment.rs"
}
```

### 19:31:44 thinking



### 19:31:44 chat

I have the incumbent's code (cycle 0008 v001, in `dash_segment.rs`). Now I'm writing the three hypotheses.

### 19:31:58 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/hypotheses/H01.md",
  "content": "# H01\n\nLetting a ghost-returning key enter its target with the CLOCK reference bit already set should raise hit-rate, mainly at `web_0` 16K: proven returnees outlast fresh promotees in full 2-slot buckets, a victim signal that LRU order does not carry.\n\n## Parent\n\ncycle/0008/v001\n\n## Changes\n\nAll edits go in `src/dash/segments/dash_segment.rs`. Keep every constant, the ghost lifecycle (factor 2, consume-on-put), the rescue, the two-hit gate with its two-free-slot bypass, `make_room_clock`, and all placement positions exactly as in v001. Admission does not change. Only the reference bit of the newly placed item changes.\n\n1. **Ghost-origin flag in the stash counter.**\n   - Add `const GHOST_ORIGIN_FLAG` as a high bit of `Item::lfu_counter`. Check the field's integer type in `src/shared/item.rs` and use e.g. `1 << 7` for `u8` or `1 << 15` for `u16`. Add `const CARRY_GHOST_HISTORY: bool = true;`.\n   - In `DashSegment::put`, a ghost hit sets `item.lfu_counter = (STASH_PROMOTION_HITS - 1) | GHOST_ORIGIN_FLAG`. A miss still sets 0.\n2. **Hit counting ignores the flag.** In the stash branch of `promotion_get`, compute the hit count on `lfu_counter & !GHOST_ORIGIN_FLAG`:\n   - add 1, saturating;\n   - store the result back with the flag preserved;\n   - compare the unflagged count against `STASH_PROMOTION_HITS`.\n\n   The gate and the bypass therefore behave exactly as in v001.\n3. **Carry the flag into the target.** On the promotion path, replace the unconditional `lfu_counter = 0` before `remove_at(item_idx)`:\n   - read `flagged = lfu_counter & GHOST_ORIGIN_FLAG != 0`;\n   - set `lfu_counter = if flagged && CARRY_GHOST_HISTORY { 1 } else { 0 }`.\n\n   `promote_to_target` does not touch the counter, and `make_room_clock` runs before the `put_at`, so the newcomer lands at MRU with bit 1. On the next promotion into that bucket, CLOCK spares it once: its bit is cleared and an unreferenced resident goes instead. After that its lifecycle is the normal one.\n4. **The flag never escapes the stash elsewhere.** These paths already assign `lfu_counter = 0`, which clears the flag: rescue in `put_stash_recording`, every `demote_from_normal_bucket` path, and `promote_to_probing`. Keep them. A flagged item evicted out of the cache is recorded in the ghost as before.\n\nTests:\n- Extend `ghost_put_promotes_on_first_stash_hit` to assert that the promoted `k` has `lfu_counter == 1` in its target.\n- Add `fresh_promotee_enters_with_bit_zero`: a non-ghost two-hit promotee has `lfu_counter == 0`.\n- Add `ghost_flag_does_not_change_gate`: a ghost-credited key in the stash promotes on exactly its first hit, and its stored stash count reads 1 when masked.\n- Keep all existing tests passing.\n\nInvariants: no duplicates, slot legality, the `get` postcondition, and ghost recording on every out-of-cache eviction. There is no new metadata, because the flag reuses a spare bit of `lfu_counter`.\n\n## Rationale (optional)\n\nc8 showed that CLOCK in 2-slot buckets agrees with LRU because every entry starts at bit 0, so the bit only repeats recency. The ghost is the only signal in the lineage that holds reuse across a stay outside the cache. Today that evidence is spent entirely on admission and then dropped. This hypothesis moves it into the victim choice. It does not loosen admission, so it avoids the recurring \"easier route into a crowded target\" failure. What it changes is which resident pays for the next promotion: a fresh one-stay promotee rather than a key that has already come back once.\n"
}
```

### 19:32:11 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/hypotheses/H02.md",
  "content": "# H02\n\nRequiring one extra stash hit before a promotion when every resident of the full target has its CLOCK reference bit set should lift hit-rate, mostly at 16K: admission then depends on the value of the resident it would displace, not on free slots.\n\n## Parent\n\ncycle/0008/v001\n\n## Changes\n\nAll edits go in `src/dash/segments/dash_segment.rs`. Keep the ghost, the rescue, `make_room_clock`, the two-free-slot bypass, and every placement position as in v001.\n\n1. **Add the constants.**\n   - `const VALUE_AWARE_GATE: bool = true;`\n   - `const REFERENCED_TARGET_EXTRA_HITS: usize = 1;`\n2. **Add a helper** `fn target_all_referenced(core, target_idx) -> bool`. It returns true when `core.buckets[target_idx].is_full()` and every item in `get_items()` has `lfu_counter != 0`. In a normal bucket the counter is the 1-bit reference flag. This is a read-only scan of 2 slots. It must not clear any bits, so the CLOCK state is untouched when a promotion is refused.\n3. **Change the stash branch of `promotion_get`.**\n   - After computing `hits`, `free` and `target_has_room`, compute `required`:\n     - `STASH_PROMOTION_HITS + REFERENCED_TARGET_EXTRA_HITS` when `VALUE_AWARE_GATE && !target_has_room && target_all_referenced(core, target_idx)`;\n     - `STASH_PROMOTION_HITS` otherwise.\n   - Replace `hits < STASH_PROMOTION_HITS` with `hits < required` in the refresh condition.\n   - Effects:\n     - A fresh key needs 3 hits to enter a target whose residents have both been hit since placement.\n     - A ghost-credited key, which starts at count 1, needs 2 hits.\n     - Whenever at least one resident is unreferenced, everything behaves as in v001. That resident is exactly the one `make_room_clock` would displace.\n   - The bypass is unchanged. With 2 or more free slots the first hit still promotes.\n4. **Keep the stash count intact on refusal.** The `saturating_add(1)` refresh path already does this. The count resets only on promotion and on every stash entry, as today. A key that reaches `required` promotes through `promote_to_target`, where `make_room_clock` clears both bits and evicts the LRU resident as before. That clearing is the decay, so a target cannot block admission forever.\n5. **Refresh `promotion_get`'s doc comment** and the struct doc to describe the value-aware gate.\n\nTests:\n- Add `referenced_full_target_requires_extra_hit`:\n  - fill target 0 with `fill_target_via_stash` and set both residents' `lfu_counter = 1` via `get_items_mut`;\n  - put a third key and hit it twice: it stays in the stash at MRU;\n  - hit it a third time: it promotes, and one former resident is now in its stash.\n- Add `unreferenced_resident_keeps_two_hit_gate`: with one resident at bit 0, two hits promote, as in v001.\n- Add `ghost_key_needs_two_hits_into_referenced_target`.\n- Keep all existing tests passing.\n\nInvariants: no duplicates, slot legality, the `get` postcondition (a refused key is returned from its stash slot), and ghost recording on every eviction out of the cache. There is no new metadata. The per-op cost is one 2-item scan on gated stash hits.\n\n## Rationale (optional)\n\nThe c8 continuation asks for admission based on how valuable the displaced resident is. All previous gate changes were keyed on occupancy, and the ones that eased entry into crowded targets lost at 16K (c3 v001/v002, c6 v003). This hypothesis tightens entry only in the one state where every possible victim has shown reuse since it was placed, which is exactly the saturated `web_0` 16K regime. It leaves admission unchanged wherever a cold resident exists, which preserves `hm_0` 128K's profitable displacements.\n"
}
```

### 19:32:24 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/hypotheses/H03.md",
  "content": "# H03\n\nSplitting the segment-wide ghost ring into per-stash-bucket sub-rings of equal total capacity should cut per-op-time by shrinking the put-path ghost scan from `2 × stash slots` to `2 × bucket_size` entries, with hit-rate within noise of the incumbent.\n\n## Parent\n\ncycle/0008/v001\n\n## Changes\n\nAll edits go in `src/dash/segments/dash_segment.rs`. No policy logic changes: the gate, bypass, rescue, CLOCK, placement and `GHOST_FACTOR = 2` all stay as in v001.\n\n1. **Restructure `GhostRing`** into `slots: Vec<u64>`, `heads: Vec<u8>` (one head per stash bucket) and `width: usize`.\n   - `width = GHOST_FACTOR * bucket_size`. Assert that `width <= u8::MAX + 1`, or use `u16` heads.\n   - `slots.len() = num_of_stash_buckets * width`, so total capacity is unchanged.\n   - Change `GhostRing::new(len)` to `GhostRing::new(num_buckets, width)` and update `DashSegment::try_new` to match.\n2. **Make the operations sub-ring local.**\n   - `insert(sb, fp)` writes `slots[sb * width + heads[sb]]` and advances `heads[sb]` modulo `width`. This is FIFO overwrite within the sub-ring.\n   - `remove(sb, fp)` scans only `slots[sb*width .. (sb+1)*width]`, zeroes the first match, and returns whether it found one.\n   - `contains(sb, fp)` is the `#[cfg(test)]` equivalent.\n3. **Change the call sites.** The bucket index is `sb = get_index(&key, core.stash_size)`, the same stash bucket the key would occupy.\n   - In `put_stash_recording`, compute it from `evicted.key` before `ghost.insert`.\n   - In `DashSegment::put`, compute it from `item.key` before `ghost.remove`. `core.stash_size` is reachable through `self.core`.\n   - The fingerprint function is unchanged.\n4. **Update the doc comments**: the metadata line, which is now `slots × 8 + num_of_stash_buckets` bytes, and the decay description. A fingerprint now survives about `GHOST_FACTOR × bucket_size` evictions from its own stash bucket, not about that many evictions segment-wide.\n\nTests:\n- Rewrite `ghost_ring_overwrites_oldest` for a single sub-ring: `new(1, 2)`, then insert 3, 5, 7 and check that 3 was overwritten.\n- Add `ghost_sub_rings_are_independent`: overflowing bucket 0's sub-ring must not evict a fingerprint stored in bucket 1.\n- Update the `seg.ghost.contains(...)` assertions in existing tests to pass the stash index of the key.\n- Keep all other tests passing unchanged.\n\nInvariants: no duplicates, slot legality, the `get` postcondition, and every out-of-cache eviction is recorded in its key's sub-ring. The consume-on-put lifecycle is unchanged.\n\n## Rationale (optional)\n\nPer-op-time and metadata-size have gone unmeasured for 8 cycles, and the ghost's linear scan on every `put` is the most obvious cost the lineage has added since baseline. Partitioning by stash bucket keeps the overall reach (factor 2) while scanning only `1/num_of_stash_buckets` of the ring. Every key's stash bucket is fixed by its hash, so a returning key always lands in the sub-ring it was recorded in, and recall does not drop. The only hit-rate effect is that reach becomes local to each bucket. Stash overflow is roughly balanced by the hash, so returns that came within one to two stash-capacities, which c7 identified as where the ghost's value lies, should be credited about as often as before.\n"
}
```

### 19:32:33 thinking



### 19:32:35 chat

I've written the three cycle 0009 hypotheses. All three use the incumbent, cycle/0008/v001 (hit-rate 0.4034), as their parent, and every change stays inside `src/dash/segments/dash_segment.rs`, away from the protected files. Each needs only a few lines of new logic and adds no metadata, except H03's one small counter per stash bucket.

- **H01:** keys that come back through the ghost enter their target with the reference bit already set. In a full 2-slot bucket, the next promotion then evicts a fresh newcomer rather than a key that has already proven it returns. Admission rules don't change, so this avoids the "easier way back into a crowded target" pattern that has lost four times. It targets the flat `web_0` 16K cell with a victim signal that plain recency order doesn't contain. That was the gap cycle 8 identified.
- **H02:** when the target is full and both residents have been hit since they were placed, a stash item needs one extra hit before it can promote. Wherever a resident hasn't been hit, it behaves exactly like v001. So entry gets stricter only in the saturated state, and `hm_0` 128K keeps the displacements that pay off there. This is cycle 8's "judge by the value of the resident being displaced" idea.
- **H03:** splits the single segment-wide ghost ring into one small ring per stash bucket, with the same total size. Each `put` then searches a few entries instead of the whole ring. This one is aimed at per-op-time, with hit-rate expected to stay within noise. It only pays off if per-op-time is actually measured. The leaderboard shows a baseline value (107), but no variant has reported one in 8 cycles.

Two things the implementer should check:
- H01 stores a marker in a high bit of `lfu_counter`. Its integer type is in `src/shared/item.rs`, which I didn't open; the right bit position depends on that type.
- H03's per-bucket counters assume a sub-ring width of 256 or less, otherwise they need a wider type.

I couldn't run shell commands or file searches here. So I read only cycle 0008's v001 code and the summary files; I didn't look at the overlays for v002 and v003. Those two variants were dropped by cycle 8's own conclusions, so the gap should be harmless.

Files are in `/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/hypotheses/`:
- `H01.md`
- `H02.md`
- `H03.md`


---

- finished: 2026-10-28T19:32:35+00:00
- duration: 128.01s
- status: ok
- input_tokens: 24
- output_tokens: 11325
- cache_read_tokens: 363932
- cache_write_tokens: 55549
- cost_usd: 0.743774
