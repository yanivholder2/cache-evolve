# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0009/hypotheses
- allow_edits: True
- started: 2026-10-20T09:18:45+00:00

## timeline

### 09:18:56 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211; cat cross-cycle-insights.md leaderboard.md cycles/0008/cycle-insight.md; cat cycles-ledger.jsonl | tail -4; for v in cycles/0008/variants/*; do echo \"== $v\"; cat $v/insight.md; cat $v/eval/fitness-vector.json; ls $v/attempts; done"
}
```

### 09:18:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cross-cycle-insights.md"
}
```

### 09:18:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/cycle-insight.md"
}
```

### 09:18:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles-ledger.jsonl"
}
```

### 09:19:03 thinking



### 09:19:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/leaderboard.md"
}
```

### 09:19:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/cycle-winners.json"
}
```

### 09:19:05 tool: Glob

```json
{
  "pattern": "cycles/0008/variants/*/attempts/*/overlay/**/*",
  "path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211"
}
```

### 09:19:07 thinking



### 09:19:08 tool: Grep

```json
{
  "pattern": "fn ",
  "path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants",
  "output_mode": "count"
}
```

### 09:19:09 thinking



### 09:19:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts"
}
```

### 09:19:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline"
}
```

### 09:19:12 thinking



### 09:19:12 chat

I'm reading the cycle-8 overlays now, so the hypotheses can name concrete functions.

### 09:19:13 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v003/insight.md"
}
```

### 09:19:14 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v003/attempts/01/overlay/src/dash/segment.rs"
}
```

### 09:19:14 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src/dash/mod.rs"
}
```

### 09:19:16 thinking



### 09:19:22 tool: Agent

```json
{
  "description": "Map overlay source files",
  "subagent_type": "Explore",
  "run_in_background": false,
  "prompt": "Read-only task. List all files (recursive, full absolute paths) under:\n1. /home/user/cache-evolve/inputs/subjects/dash/baseline\n2. /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts (and v002, v003) — report the latest attempt number for each and the overlay file paths.\n3. /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/variants/v002/attempts — latest attempt overlay file paths.\n\nThen, for the cycle 0008 v001 latest overlay (the single-hash latency parent, built on cycle 0007 v002), give me a precise structural summary: the names and signatures of the key types (Item, bucket type(s), segment type, eviction policy trait), how bucket slots are stored (Vec<Item>? Box<dyn Bucket>? policy boxed?), bucket sizes / number of slots for normal (target/probing) and stash buckets, number of buckets per segment, the get() lookup sequence (s0, target, probing, s1 with borrowed/foreign gates), the put() path, the stash insert/eviction functions, promote/demote functions, and where `borrowed` and `foreign` counters live. Quote function names and file paths exactly. Also briefly state how cycle 0008 v002 (target-only rescue) differs from v001's parent (which function, what logic), and what the eviction policy implementations are (LRU via position ordering? Front/Back semantics). Keep the report under ~900 words, but precise."
}
```

### 09:19:25 tool: Bash

```json
{
  "command": "find /home/user/cache-evolve/inputs/subjects/dash/baseline -type f; O=/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles; for v in v001 v002 v003; do echo \"== 0008 $v\"; ls $O/0008/variants/$v/attempts; done; echo \"== 0007 v002\"; ls $O/0007/variants/v002/attempts",
  "description": "List baseline files and attempt dirs"
}
```

### 09:19:28 tool: ToolSearch

```json
{
  "query": "select:Glob,Grep",
  "max_results": 5
}
```

### 09:19:31 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline"
}
```

### 09:19:31 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts"
}
```

### 09:19:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/Cargo.toml"
}
```

### 09:19:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/001"
}
```

### 09:19:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/0001"
}
```

### 09:19:37 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/1"
}
```

### 09:19:37 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01"
}
```

### 09:19:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/a001"
}
```

### 09:19:40 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/02"
}
```

### 09:19:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v002/attempts/02"
}
```

### 09:19:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v003/attempts/02"
}
```

### 09:19:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/variants/v002/attempts/02"
}
```

### 09:19:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay"
}
```

### 09:19:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src"
}
```

### 09:19:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v002/attempts/01/overlay"
}
```

### 09:19:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v003/attempts/01/overlay"
}
```

### 09:19:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/variants/v002/attempts/01/overlay"
}
```

### 09:19:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline/src"
}
```

### 09:19:49 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/lib.rs"
}
```

### 09:19:49 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash.rs"
}
```

### 09:19:50 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/main.rs"
}
```

### 09:19:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/variant.json"
}
```

### 09:19:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/mod.rs"
}
```

### 09:19:54 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/segment.rs"
}
```

### 09:19:54 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/bucket.rs"
}
```

### 09:19:55 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/item.rs"
}
```

### 09:19:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash"
}
```

### 09:19:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/src"
}
```

### 09:20:02 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/dash.rs"
}
```

### 09:20:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/eviction.rs"
}
```

### 09:20:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/lib.rs"
}
```

### 09:20:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/dash_table.rs"
}
```

### 09:20:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/table.rs"
}
```

### 09:20:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/policy.rs"
}
```

### 09:20:08 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/diff.patch"
}
```

### 09:20:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/patch.diff"
}
```

### 09:20:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/result.json"
}
```

### 09:20:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/hypothesis.md"
}
```

### 09:20:11 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/subject.json"
}
```

### 09:20:11 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/README.md"
}
```

### 09:20:15 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/segment.h"
}
```

### 09:20:16 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/dash.h"
}
```

### 09:20:16 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/Segment.java"
}
```

### 09:20:17 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/segment.go"
}
```

### 09:20:18 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/segment.py"
}
```

### 09:20:18 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/__init__.py"
}
```

### 09:20:34 thinking



### 09:20:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/dash_segment.rs"
}
```

### 09:20:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/segment_core.rs"
}
```

### 09:20:36 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/promotion.rs"
}
```

### 09:20:36 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/insight.md"
}
```

### 09:20:37 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v002/insight.md"
}
```

### 09:20:40 thinking



### 09:20:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/core.rs"
}
```

### 09:20:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/dash_bucket.rs"
}
```

### 09:20:43 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0008/variants/v001/attempts/01/overlay/src/dash/segment/mod.rs"
}
```

### 09:20:43 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0007/variants/v002/insight.md"
}
```

### 09:21:06 thinking



### 09:21:18 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0009/hypotheses/H01.md",
  "content": "# H01\n\nStoring every bucket's slots inline in one flat per-segment item array, with a statically dispatched position-ordered policy, will lower get p50 below 69 ns while keeping hit-rate exactly at 0.7095.\n\n## Parent\n\ncycle/0007/v002\n\n## Changes\n\nChange only the storage layout inside `SegmentCore` (and the bucket code it owns). The observable cache behaviour must not change at all.\n\n- Remove the per-bucket `Vec<Item>` and the per-bucket `Box<dyn EvictionPolicy>`, and drop `Vec<Box<dyn Bucket>>` as the container. In their place, `SegmentCore` holds two things for the normal buckets (target/probing): one contiguous `Vec<Item<K,V>>` of length `num_normal_buckets * NORMAL_SLOTS`, and a matching `Vec<u8>` of per-bucket lengths. It holds the same pair for the stash buckets, sized `num_stash_buckets * STASH_SLOTS`. Bucket `i` owns the slice `[i*N .. i*N + len[i]]`. Use the bucket capacities the parent already uses, and allocate each array once in the segment constructor.\n- Add free functions or inherent methods on `SegmentCore` that work on a `&mut [Item]` slice plus its `&mut u8` length: `find_in(slice, key) -> Option<usize>`, `touch(slice, pos)`, `insert_at(slice, len, item, Front|Back)`, `remove_at(slice, len, pos)`, and `victim(slice, len)`. Each must reproduce the parent's `EvictionPolicy` semantics exactly: the same position ordering, the same recency move on hit in `get_from_position`, the same Front/Back meaning in `put_at`, and the same victim index in `evict_item`. They are plain `#[inline]` functions, with no trait objects.\n- Rewrite `promotion_get`, `promote_to_target`, `demote_from_normal_bucket`, `stash_insert_known`, `put_stash_at_index` and the put path to call these helpers on slices of the flat arrays instead of the bucket trait methods. Keep the lookup order `s0 → target → (probing if foreign[probing_idx] != 0) → (s1 if borrowed[s1] != 0)`. Keep every `borrowed`/`foreign` increment and decrement at the same logical event.\n- Do not add fields to `Item`, and do not grow the `Dash` struct. The new arrays live behind `SegmentCore`'s heap allocation, so metadata-size stays at 40/24.\n- If the `Bucket` trait is still needed for compilation by files outside the authoring scope, leave it defined but unused on the hot path.\n\n## Rationale (optional)\n\nCycle 8 showed that the median get no longer responds to fewer hashes (v001) or to removing the `Box<dyn Bucket>` hop (v003). What remains is two dependent loads per probe (bucket header → heap item buffer) plus a vtable call into the boxed policy on hits. A flat array makes the location of bucket `i`'s slots computable from `i` alone, which removes both. Pass/fail criteria: hit-rate must equal 0.7095 exactly (the change is state-neutral), and get p50 must drop below 69 ns. If p50 stays flat, slot walking itself is the floor, not indirection.\n"
}
```

### 09:21:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0009/hypotheses/H02.md",
  "content": "# H02\n\nAdding a one-byte key tag per slot to the `DashBucket` header will let lookups reject non-matching buckets without dereferencing the item buffer, which should lower get p50 and p99 while leaving hit-rate exactly at 0.7095.\n\n## Parent\n\ncycle/0008/v003\n\n## Changes\n\nKeep v003's concrete `DashBucket` stored contiguously in `Vec<DashBucket>` inside `SegmentCore`, and add an inline tag array to each header.\n\n- Add `tags: [u8; MAX_SLOTS]` to `DashBucket`, where `MAX_SLOTS` is the largest bucket capacity the parent uses (normal or stash). The tag is a byte taken from the key hash that the lookup has already computed, e.g. `(h >> 56) as u8`. Pick bits that are not used for segment, bucket or stash indexing, and add no new hash. Keep `tags[j]` in sync with the item at position `j` in the bucket's `Vec<Item>`. Every mutation must shift the tag array the same way it shifts the items: `put_at` (Front/Back insert), `remove_at`, `evict_item`, and the recency reorder in `get_from_position`. Paths that insert an item without a hash in hand (demotion, cascades) must get the hash passed in from the caller. If no hash is available, the caller computes `key_hash` once at that point.\n- In `DashBucket::get_position`, compare `tags[0..len]` against the probe tag first. Only for slots whose tag matches, load the `Item` from the heap buffer and compare the full key. If no tag matches, return `None` without touching the item buffer.\n- Thread the probe tag from `promotion_get` into each bucket probe (`s0`, target, the gated probing bucket, the gated `s1`), and do the same on the put/exists path.\n- To offset the stride growth v003 suffered from, shrink the header where it costs nothing. For example, store `max_size` as `u8` next to the tags, so the header grows by at most a few bytes. `Item` and the `Dash` struct must not change, so metadata-size stays 40/24.\n- Behaviour must stay identical: tags only filter. A tag match always falls through to the full key compare.\n\n## Rationale (optional)\n\nv003's analysis named the item-buffer miss as the cost that remains after the bucket became concrete. Most probes on the common path end in \"not here\": a miss on `s0` before a target hit, and every probe of a full miss. With tags inline in the header, which the probe already loads, those probes never chase the `Vec<Item>` pointer. This is a different lever from H01's layout rewrite: it keeps the heap buffers and only avoids touching them. Pass/fail criteria: hit-rate must equal 0.7095, and get p50 must fall below 70 ns. If p50 is flat while p99 falls, item-buffer misses are a tail effect only.\n"
}
```

### 09:21:38 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0009/hypotheses/H03.md",
  "content": "# H03\n\nDoubling the slots per stash bucket and halving the number of stash buckets, at the same total segment capacity, will raise hit-rate above 0.7095 by letting the probationary queue hold reuse candidates longer.\n\n## Parent\n\ncycle/0006/v002\n\n## Changes\n\nChange only the stash geometry in the segment constructor of the parent, plus the index math that depends on it.\n\n- Find the constants or constructor arguments in `SegmentCore` / `DashSegment` that set the number of stash buckets and the slots per stash bucket (currently 2-slot stash buckets). Set the slots per stash bucket to twice the current value and the number of stash buckets to half the current value, so that total stash slots, and therefore total cache capacity at size 2048, stay exactly the same. Do not change normal (target/probing) buckets.\n- Update every function that maps a key to a stash bucket index (the `s0` computation and the `s1 = (s0 + 1) % num_stash_buckets` neighbour) so it uses the new stash bucket count. Keep two-choice placement: fill a free slot in `s0`, then `s1`, before evicting. Keep the `s1`-probed-last lookup order and the `borrowed` counter gate. Size the `borrowed` counter type so it can count up to the new slots per bucket.\n- Keep the stash ordering semantics unchanged within the larger buckets: fresh puts go in at the LRU end (Front), demotees go in at the MRU end (Back), a stash hit promotes to the target at Front, and the victim is the LRU-end item.\n- Do not change `Item` or the `Dash` struct; metadata-size stays 40/24.\n\n## Rationale (optional)\n\nEvery hit-rate gain so far has come from giving the stash more effective capacity per key (two-choice, rescue), and cycle 3 attributed the failure of reuse-based filtering to tiny 2-slot buckets that evict before a second touch can arrive. Stash size and associativity are the main structural lever never tested (an open question since cycle 3). Wider buckets pool overflow across more keys, the same pooling effect two-choice gave, without any new hashing or rescue cascades into target buckets. That makes this the rescue-free alternative for hit-rate. Pass/fail criteria: the hypothesis is confirmed if hit-rate rises above 0.7095 (towards the 0.711–0.713 range of rescue variants) while get p50 stays within about 5 ns of the parent's 72 ns. It is refuted if hit-rate is flat or lower, or if longer stash scans add more than the hit-rate is worth.\n"
}
```

### 09:21:47 thinking



### 09:21:47 chat

I wrote the three cycle-0009 hypotheses to `cycles/0009/hypotheses/`. I couldn't open any source code, though. This session only allowed reading files at exact paths, and none of my guessed file names exist, even though the overlay directories do. So the function names in each `## Changes` section (`promotion_get`, `put_at`, `stash_insert_known`, `SegmentCore`, `DashBucket`, …) come from the insight files, not from the code itself. The implementer may need to adjust them.

| File | Parent | Target | Idea |
|---|---|---|---|
| H01 | cycle/0007/v002 (incumbent) | get p50, hit-rate exactly 0.7095 | Store all of a segment's bucket slots in one flat array and replace the boxed policy with plain inline functions. This removes both pointer hops per probe. It is the step cycle 8 recommended. |
| H02 | cycle/0008/v003 (incumbent) | get p50 and p99, hit-rate exactly 0.7095 | Add a one-byte key tag per slot to the bucket header. Probes that find no matching tag skip the separate item buffer entirely. Tags come from hash bits already computed, so `Item` and `Dash` stay the same size. |
| H03 | cycle/0006/v002 (incumbent) | hit-rate above 0.7095 | Double the slots per stash bucket and halve the number of stash buckets, keeping total capacity the same. Stash size has been an untested open question since cycle 3, and this raises hit-rate without rescue's cost. |

- **Requirements:** all three cite an incumbent, and all keep metadata-size at 40. None touches the protected files.
- **Distinct strategies:** H01 changes the data layout. H02 filters probes but keeps the existing layout. H03 changes the stash shape.
- **What to check in H03:** I took the "2-slot" stash from the insights and the Front-is-LRU-end mapping from the cycle 1–2 notes. Neither is confirmed in code.


---

- finished: 2026-10-20T09:21:47+00:00
- duration: 181.76s
- status: ok
- input_tokens: 22
- output_tokens: 9783
- cache_read_tokens: 349854
- cache_write_tokens: 50473
- cost_usd: 0.949363
