# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007/hypotheses
- allow_edits: True
- started: 2026-10-21T09:35:41+00:00

## timeline

### 09:35:45 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878; cat leaderboard.md; cat cross-cycle-insights.md 2>/dev/null; cat cycles-ledger.jsonl; cat cycles/0006/cycle-insight.md; cat cycles/0006/variants/*/insight.md; ls cycles/0006/variants/*/attempts/*/overlay -R | head -50; find /home/user/cache-evolve/inputs/subjects/tiny_lfu/baseline -type f"
}
```

### 09:35:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/leaderboard.md"
}
```

### 09:35:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cross-cycle-insights.md"
}
```

### 09:35:48 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles-ledger.jsonl"
}
```

### 09:35:48 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0006/cycle-insight.md"
}
```

### 09:35:50 thinking



### 09:35:50 tool: Glob

```json
{
  "pattern": "cycles/0006/variants/*/**/*",
  "path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878"
}
```

### 09:35:51 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/baseline"
}
```

### 09:35:52 thinking



### 09:35:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0006/cycle-winners.json"
}
```

### 09:35:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0006/variants/v003/attempts/01/overlay/src/tiny_lfu/mod.rs"
}
```

### 09:36:15 thinking



### 09:36:25 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007/hypotheses/H01.md",
  "content": "# H01\n\nLetting a newcomer win a tie when it was evicted recently (a small ghost set of evicted key hashes) will raise hit-rate on workload-a by re-admitting keys whose reuse distance is just above the cache size.\n\n## Parent\n\ncycle/0006/v003\n\n## Changes\n\nAdd a non-resident \"ghost\" history to `TinyLfuCache` in `src/tiny_lfu/mod.rs`. It records the 64-bit hashes of keys that were recently evicted, using the same `hash_key` value already computed in `put`.\n\n- New fields: `ghost_ring: Vec<u64>` (fixed length `GHOST_CAP = capacity`, starting at 0), `ghost_pos: usize`, and `ghost_set: HashSet<u64, FastBuildHasher>` (pre-sized to `capacity`). Allocate them in `try_new`.\n- On eviction in `put`, inside the `incoming_freq > victim_freq` branch and before the victim's key is overwritten, compute `hash_key(&victim_key)` and insert it into the ring at `ghost_pos`. If the ring slot already held a non-zero hash, first remove that old hash from `ghost_set`. Insert the new hash into `ghost_set` and advance `ghost_pos` modulo `GHOST_CAP`. Map a hash of 0 to 1 so that 0 can stay the \"empty\" sentinel.\n- Change the admission contest only for ghost hits. Let `is_ghost = ghost_set.contains(&key_hash)`. Admit if `incoming_freq > victim_freq || (is_ghost && incoming_freq == victim_freq && incoming_freq >= 2)`. When a ghost key is admitted, remove its hash from `ghost_set` but leave the ring slot alone, because a stale ring entry is harmless: it only triggers a removal that does nothing. Everything else stays the same. Non-ghost ties still keep the victim (strict `>`), victim sampling (`select_victim`, `VICTIM_SAMPLE = 8`) is unchanged, demotion (`select_demotion`) is unchanged, and dedup and the 16× sketch are unchanged.\n- Rejected newcomers are not added to the ghost set. Only keys that actually lived in the cache are added.\n- Invariants: `check_structure` must still hold, and `map.len() == nodes.len() <= capacity`. Ghost memory is bounded at `capacity` u64 values plus the set. Add a unit test showing that an evicted key which comes back and ties the victim's estimate gets admitted, while a never-seen key with the same estimate is rejected.\n\n## Rationale\n\nEvery earlier attempt to loosen the contest applied tie-admission to all newcomers and swapped one cold key for another. A ghost hit is evidence the earlier tests never used: the key was resident recently, so it has a real reuse pattern of roughly the cache size. LIRS-style traces are built around exactly this case. The `>= 2` guard keeps one-hit wonders out.\n"
}
```

### 09:36:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007/hypotheses/H02.md",
  "content": "# H02\n\nAdmitting warm candidates that lose the contest with a small probability (Caffeine-style jitter, 1/128 when the candidate's estimate is above 5) will raise hit-rate on workload-a, because it breaks the lock-in of residents with stale-high lifetime counts without adding aging.\n\n## Parent\n\ncycle/0005/v001\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs` of the parent (the SLRU with plain LRU demotion and probation-only 8-way min-estimate victim sampling), change only the admission decision in `put`.\n\n- Add a field `rng: u64` to `TinyLfuCache` and seed it in `try_new` with a fixed constant (e.g. `0x9E37_79B9_7F4A_7C15`) so that runs are deterministic. Add a helper `fn next_rand(&mut self) -> u64` that advances `rng` by the golden-ratio increment and returns `splitmix64(rng)`. Reuse the existing `splitmix64`.\n- Replace `if incoming_freq > victim_freq { ...admit... }` with: admit if `incoming_freq > victim_freq`. Otherwise, if `incoming_freq >= JITTER_MIN_FREQ` (const `6`) and `self.next_rand() & 127 == 0`, admit anyway. Otherwise reject as before. Only call the RNG when the first test fails and the candidate is warm, so cold misses never use RNG state. Note that this is not tie-admission: one-hit and low-count newcomers (estimate ≤ 5) still lose every tie and every lower comparison.\n- The admit path (unlink the victim, fix `prot_len`, reuse the slot, `push_back` into probation, update the map) is unchanged. Do not touch the sketch width, dedup, `select_victim`, or the SLRU promotion and demotion logic.\n- Add a unit test showing that a candidate with estimate ≤ 5 that ties the victim is never admitted over many trials, while a candidate with estimate ≥ 6 that loses is eventually admitted (within a few thousand trials).\n\n## Rationale\n\nThe cycle history says lifetime counts are the right signal and aging always hurt. The cost of lifetime counts is that a warm key repeatedly loses to residents whose counts reflect old popularity. Rare random admission of keys that are demonstrably warm adds just enough churn to let such keys in, without the wholesale freshness that aging imposes. This lever acts on the contest, not on the sketch, the window or SLRU internals, so it is distinct from every earlier lineage.\n"
}
```

### 09:36:43 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0007/hypotheses/H03.md",
  "content": "# H03\n\nShrinking the SLRU protected segment from 80% to 50% of capacity will raise hit-rate on workload-a, because admitted newcomers get a longer probation trial in which to earn the second hit that promotes them.\n\n## Parent\n\ncycle/0006/v003\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, change the size of the protected segment in `try_new` from `(capacity * 8 / 10).max(1)` to `(capacity / 2).max(1)`. To keep this easy to sweep, add a const `PROTECTED_NUM: usize = 5` / `PROTECTED_DEN: usize = 10` and compute `(capacity * PROTECTED_NUM / PROTECTED_DEN).max(1)`.\n\nEverything else stays exactly as in the parent:\n- strict `>` admission\n- dedup of the get-miss-then-put increment\n- the 16× sketch\n- probation-only 8-way min-estimate victim sampling in `select_victim`\n- frequency-aware demotion over the 4 oldest protected entries in `select_demotion`\n\nUpdate the unit tests that hard-code the 80% ratio. `protected_overflow_demotes_to_probation` uses capacity 5 → `protected_cap` 2. `protected_overflow_spares_hot_head_and_demotes_colder_entry` uses capacity 10 → `protected_cap` 5. Adjust their expected `prot_len`, and the keys whose promotion causes overflow, so the tests keep checking the same behaviour: overflow demotes the lowest-estimate entry among the oldest protected entries, falling back to LRU on ties. `check_structure` invariants must still hold.\n\n## Rationale\n\nThe windows tried earlier failed because they skip admission and give only about 20 slots of recency trial. Probation is different: it is behind the frequency gate, so only keys that won the contest get the longer trial. At capacity 2048, an 80% protected cap leaves about 410 probation slots. A newcomer that won admission is then sampled for eviction after about 410 further admissions, which may be shorter than the reuse distance of LIRS-trace keys. The probation ratio was raised as an open question in cycle 5 and has never been tested. Unlike the demotion and window tweaks, it changes how long newly admitted keys survive, not only the order of entries in the protected segment.\n"
}
```

### 09:36:51 thinking



### 09:36:52 chat

I've written the three hypotheses for cycle 0007 to the output directory as H01.md, H02.md and H03.md. All three aim at hit-rate, because hit-rate is still the only dimension being scored. Two build on the cycle/0006/v003 incumbent and one on the cycle/0005/v001 incumbent, so both current winners are used as parents.

| # | Parent | Idea | Why it's different from earlier tries |
|---|---|---|---|
| H01 | cycle/0006/v003 | Keep a small "ghost" list of the hashes of keys that were recently evicted, sized to the cache. If a returning key is on that list and ties the victim with an estimate of at least 2, admit it. | Earlier tie-admission applied to every newcomer and only swapped one cold key for another. Here only keys that were recently in the cache get the tie, which matches the reuse pattern LIRS-style traces are built around. |
| H02 | cycle/0005/v001 | When a warm newcomer (estimate of 6 or more) loses the admission contest, admit it anyway 1 time in 128, using a seeded random number generator so runs are repeatable. | This loosens the lock-in from never-aging counts without adding aging, which has failed every time it was tried. Low-count newcomers are still always rejected on ties. |
| H03 | cycle/0006/v003 | Shrink the protected segment from 80% to 50% of the cache. | Admitted newcomers get a longer probation trial (about 1024 slots instead of about 410) to earn a second hit. Unlike the failed window, this space is behind the frequency gate. The probation ratio was raised as an open question in cycle 5 and never tested. |

**Conflict with cycle 6's advice:** H03 is a change inside the SLRU, and the cycle 6 summary said to stop spending variants on those. I kept it because it changes how long admitted keys survive, not just how entries are reordered (demotion order and windows were what failed). If you'd rather follow that advice strictly, H03 is the one to swap out.

**What I couldn't read:** I couldn't list files in this session, so I opened only cycle 6 v003's code (found by guessing its path) and the summary files. I didn't see the code for cycle 6 v001/v002 or for cycle/0005/v001. H02 describes its parent (the SLRU with plain least-recently-used demotion) from the cycle summaries rather than its source.


---

- finished: 2026-10-21T09:36:52+00:00
- duration: 71.15s
- status: ok
- input_tokens: 12
- output_tokens: 6855
- cache_read_tokens: 157341
- cache_write_tokens: 30389
- cost_usd: 0.411728
