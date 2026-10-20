# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006/hypotheses
- allow_edits: True
- started: 2026-10-20T20:12:16+00:00

## timeline

### 20:12:22 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589; cat leaderboard.md cross-cycle-insights.md cycles/0005/cycle-insight.md; tail -c 4000 cycles-ledger.jsonl; ls cycles/0005/variants/*/attempts/; for v in cycles/0005/variants/*; do echo $v; cat $v/insight.md; done; find /home/user/cache-evolve/inputs/subjects/tiny_lfu/baseline -type f"
}
```

### 20:12:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cross-cycle-insights.md"
}
```

### 20:12:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/leaderboard.md"
}
```

### 20:12:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/cycle-insight.md"
}
```

### 20:12:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles-ledger.jsonl"
}
```

### 20:12:29 thinking



### 20:12:30 tool: Glob

```json
{
  "pattern": "cycles/0005/variants/*/attempts/*/overlay/**/*",
  "path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589"
}
```

### 20:12:31 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/baseline"
}
```

### 20:12:33 thinking



### 20:12:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/cycle-winners.json"
}
```

### 20:12:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/variants/v002/insight.md"
}
```

### 20:12:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/variants/v002/attempts/01/overlay/src/tiny_lfu/mod.rs"
}
```

### 20:13:07 thinking



### 20:13:08 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs",
  "limit": 240,
  "offset": 100
}
```

### 20:13:12 thinking



### 20:13:12 chat

I've read the context (incumbents, cycle 0005 overlays, cross-cycle insights) and I'm now writing the three hypothesis files.

### 20:13:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006/hypotheses/H01.md",
  "content": "# H01\n\nReplacing the doubly-linked LRU resident list with a CLOCK ring that marks hits with a reference bit will cut get per-op-time and shrink per-entry metadata from 24 to 16 bytes while keeping hit-rate within noise.\n\n## Parent\n\ncycle/0005/v001\n\n## Changes\n\nRemove `prev`/`next` from `Node` so `Node<u64,u64>` is just `{ key, value }` (16 bytes); update the in-file `node_is_compact` test to assert 16. Delete `unlink`, `push_back`, `head`, `tail`.\n\nAdd to `TinyLfuCache` a reference-bit vector `refbits: Vec<u64>` (one bit per slot, `ceil(capacity/64)` words) and a `hand: u32` clock pointer. Nodes stay in `nodes: Vec<Node>` indexed by the `u32` stored in `map`, as now.\n\n- `get` hit: after `increment_only`, set `refbits[idx>>6] |= 1 << (idx & 63)` and return the value. No relinking, no writes to `nodes`.\n- `put` on an existing key: overwrite the value and set the ref bit (same as a hit).\n- `put` while not full: push the node, insert into `map`, leave its ref bit **clear** (a new entry has not been re-referenced yet; this mirrors it being at the MRU end only once).\n- `put` while full: select the victim with the clock sweep — starting at `hand`, while the slot's ref bit is set, clear it and advance `hand = (hand + 1) % capacity`; the first slot with a clear bit is the victim. Do not advance `hand` past the victim yet. Run the existing duel unchanged (`incoming_freq > victim_freq`, victim estimate via `estimate_hash(key_hash(&victim.key))`). If the newcomer wins: remove the victim key from `map`, overwrite the slot in place with the new key/value, insert into `map`, leave its ref bit clear, and advance `hand` by one. If the newcomer loses: leave the victim and `hand` where they are (the victim stays the next candidate, matching LRU where a rejected newcomer leaves the LRU head unchanged).\n- Bound the sweep: it terminates within at most `capacity + 1` steps because each step clears a bit; no extra guard needed, but add a `debug_assert!`.\n\nKeep the sketch (u16, 2× width, fused conservative update, `increment_only` on `get`) and `key_hash` exactly as in the parent. Keep the existing behavioural tests passing (`eviction_prefers_hot_items`, `low_frequency_new_item_is_rejected`, `update_promotes_existing_item_and_admits_hot_arrivals`); if any depends on strict LRU ordering for capacity 2, verify the CLOCK victim choice matches and adjust only test commentary, not assertions, unless the assertion is purely LRU-order-specific.\n\n## Rationale (optional)\n\nCross-cycle insights say per-op cost likely now sits outside the sketch, including relinking on every hit, and that plain LRU is the structure to beat cheaply. CLOCK turns a hit's three-node pointer rewrite into a single bit-OR on a tiny, cache-resident bitmap, and drops 8 bytes per node. LRU-end victims already have near-equal estimates (c2), so CLOCK's coarser recency order should barely move admission outcomes.\n"
}
```

### 20:13:44 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006/hypotheses/H02.md",
  "content": "# H02\n\nReplacing the std `HashMap<K, u32>` with an in-house open-addressing index table probed by the existing `key_hash` will lower get and put per-op-time by computing one hash per operation for both lookup and sketch, with hit-rate unchanged bit for bit.\n\n## Parent\n\ncycle/0005/v001\n\n## Changes\n\nRemove `map: HashMap<K, u32, FxBuild>` (and the now-unused `FxBuild` alias if nothing else uses it; keep `FxLikeHasher` since `key_hash` uses it). Add an index table:\n\n- `slots: Vec<u32>` of length `table_size = (capacity * 2).next_power_of_two()` (load factor ≤ 0.5), initialised to `NIL`; `mask = table_size - 1`.\n- Each slot holds a node index; the key itself lives only in `nodes[idx].key` (no second copy of `K` as the std map stores).\n- `find(h, key) -> Option<(slot, idx)>`: linear probe from `(h >> 32) as usize & mask` (use the high half so the slot bits are not the same bits as sketch `h1`), stop at `NIL`; compare `nodes[idx].key == *key`.\n- `insert_slot(h, idx)`: probe to the first `NIL` and store `idx`.\n- `remove_slot(slot)`: backward-shift deletion (Knuth algorithm R): walk forward from `slot`, moving any entry whose home position does not lie cyclically in `(empty, current]` back into the hole, until reaching `NIL`. Computing an entry's home needs its hash: recompute `key_hash(&nodes[idx].key)` for each shifted entry (runs are short at load 0.5).\n\nRewrite the call sites to compute `let h = key_hash(key)` once and reuse it:\n- `get`: `self.sketch.increment_only(h); let (_, idx) = self.find(h, key)?;` then relink as today.\n- `put`: `h` feeds `increment_hash(h)` and `find(h, &key)`; on a cold insert use `insert_slot(h, idx)`; on eviction, compute `vh = key_hash(&victim.key)` once and use it both for `estimate_hash(vh)` and to locate the victim's slot (`find(vh, &victim.key)`), then `remove_slot` it, overwrite the node, and `insert_slot(h, victim_idx)`. This also removes the `victim_key.clone()`.\n\nDo not change the LRU list, `Node` layout (stays 24 bytes), the sketch, or the duel rule. Add in-file unit tests: insert/remove churn of > capacity distinct keys keeps every resident findable and every evicted key absent; and a forced-collision test (many keys sharing home slot bits) exercising backward-shift deletion.\n\n## Rationale (optional)\n\nToday each `get` hashes the key twice (Fx inside std `HashMap` plus `key_hash` for the sketch), and an evicting `put` does lookup + remove + insert in the std map plus a key clone. The sketch is exhausted as a lever; this targets the open question of whether the remaining per-op cost is in map lookup and hashing. A `u32`-slot table also takes far less memory than the std map's `(K, u32)` buckets plus control bytes.\n"
}
```

### 20:14:01 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006/hypotheses/H03.md",
  "content": "# H03\n\nPutting a 10% LRU admission window ahead of the TinyLFU-guarded main LRU, with a duel that does no extra re-hashing, will raise hit-rate above 0.7423 on workload-a by keeping short recency bursts that the strict frequency duel currently rejects.\n\n## Parent\n\ncycle/0005/v002\n\n## Changes\n\nKeep v002's sketch (u16, 2× width, conservative update) and its single-count-per-access fix (`last_miss_hash`/`last_miss_valid`). Also fold in cycle/0005/v001's fused `increment_hash`/`estimate_hash` (four explicit loads in registers, `get_unchecked`), since it leaves behaviour identical.\n\nSplit residents into two LRU lists over the same `nodes` arena: a window list (`w_head`, `w_tail`, `w_len`) and a main list (`m_head`, `m_tail`, `m_len`). Set `window_cap = max(1, capacity / 10)` and `main_cap = capacity - window_cap`. Record each node's region in the top bit of its `next` link (`REGION_BIT = 1 << 31`, link value masked with `!REGION_BIT`), as cycle/0005/v003 did, so `Node<u64,u64>` stays 24 bytes. `capacity` must then be `< 2^31`; reject larger in `try_new`.\n\n- `get` hit: relink within the node's own list (read the region bit to choose head/tail fields). No promotion between regions on hit.\n- `put` on an existing key: update value, relink within its list.\n- `put` new key: always insert at the window MRU end (the sketch increment/estimate path from v002 stays as is to produce `incoming_freq`, but it is now used only for the overflow duel below). If `w_len > window_cap` after insertion, take the window LRU node `c` as the candidate:\n  - If `m_len < main_cap`, move `c` to the main MRU end (flip its region bit).\n  - Otherwise duel `c` against the main LRU head `v`: compute `estimate_hash(key_hash(&c.key))` and `estimate_hash(key_hash(&v.key))` — exactly two `key_hash` calls on the overflow path, no more. If `freq(c) > freq(v)`, evict `v` (remove from `map`, unlink) and move `c` to main MRU; else evict `c`. Ties go to the resident `v`, as today.\n  - Reuse the evicted node's slot for the new arrival so `nodes` never exceeds `capacity`: allocate the incoming node into the freed index (insert into `map` after the eviction is decided), or while still filling, push a new node.\n- While the cache is not yet full, fill the window first, then overflow into main without duels.\n\nUpdate the in-file behavioural tests only if they encode the old no-window admission order at capacity 2 (with `window_cap = 1`); prefer keeping their assertions by noting the window holds the newest key. Add one test showing a burst key re-referenced while in the window survives a scan of one-hit keys.\n\n## Rationale (optional)\n\nThe ~1% window in c5 was judged too small to hold recency bursts and paid for repeated re-hashing; insights explicitly leave a 5–20% window as the untested open question. Sketch accuracy and duel inputs are now exhausted, so a structural admission change is the remaining hit-rate lever. Starting from v002 gives the best hit rate so far (0.7423) as the bar to beat.\n"
}
```

### 20:14:11 thinking



### 20:14:11 chat

I wrote three hypotheses for cycle 0006 into `cycles/0006/hypotheses/`. Each takes a different approach. Two build on the incumbent cycle/0005/v001, which meets the rule that at least one must cite a cross-cycle winner.

- **H01 (parent cycle/0005/v001): targets per-op-time and metadata-size.** It replaces the doubly-linked LRU list with a CLOCK ring. A hit just sets a bit in a small bitmap instead of moving the entry in the list. Removing the `prev`/`next` links shrinks each `Node` from 24 to 16 bytes. The admission duel and the sketch stay the same. The bet is that the victims CLOCK picks have about the same frequency estimates as LRU's, so hit-rate shouldn't move.
- **H02 (parent cycle/0005/v001): targets per-op-time.** It replaces the standard `HashMap` with a custom lookup table keyed by the existing `key_hash`. Each operation then hashes the key once for both the lookup and the sketch, where it now hashes twice. The key is stored once, and eviction no longer clones the victim's key. Hit-rate should be identical bit for bit. This tests the open question of whether the remaining per-op cost is in map lookup and hashing.
- **H03 (parent cycle/0005/v002): targets hit-rate.** It adds a recency window holding 10% of capacity in front of the main LRU list. New keys enter the window first, and only the overflow duel uses the frequency sketch, with at most two hash calls per duel. It folds in v001's faster sketch increment and stores the region flag in a spare link bit, so `Node` stays 24 bytes. This answers the open question about a 5–20% window after the ~1% window failed in cycle 0005. The bar is v002's 0.7423, the best hit-rate so far. v002 isn't an incumbent, probably because of its slower per-op readings, which were never confirmed as real.

H01 changes the in-file `node_is_compact` test from 24 to 16 bytes. That test lives in the editable cache file, not the locked `tests/` folder.


---

- finished: 2026-10-20T20:14:12+00:00
- duration: 115.60s
- status: ok
- input_tokens: 14
- output_tokens: 7985
- cache_read_tokens: 200875
- cache_write_tokens: 34269
- cost_usd: 0.474342
