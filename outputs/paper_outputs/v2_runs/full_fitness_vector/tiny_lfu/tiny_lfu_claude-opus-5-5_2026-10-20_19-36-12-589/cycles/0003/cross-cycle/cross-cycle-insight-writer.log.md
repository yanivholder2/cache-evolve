# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:57:54+00:00

## timeline

### 19:58:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that only remove work or shrink layout, with no new hot-path logic, are the most reliable gains. Fast hashing (cycle 1) and compact `u32` links (cycle 2) both kept behaviour unchanged and improved latency or memory.\n- Hit rate moves only when the frequency estimates get more accurate. Conservative update (cycle 3) is the only hit-rate gain so far. Every other admission, structure or victim change left hit rate flat or made it worse.\n- The lineage has a clean chain of bases: fast hash → compact links → conservative update. Each step kept the previous wins.\n\n## Recurring failures\n\n- Adding resident-list structure (window + SLRU in cycle 1, LRU-end victim sampling in cycle 2) costs latency and memory and gives no hit-rate gain. The sketch signal is the bottleneck, not the list structure.\n- Changing the admission rule or counter resolution on its own fails. Aging with a coarser cap created more ties and rejected good newcomers (cycle 2). Letting newcomers win ties also hurt (cycle 3), so the strict `>` rule is not the problem.\n- Sketch layouts that break row independence (the blocked/cache-line sketch in cycle 3) inflate estimates and hurt admission. The sketch already fits in cache, so layout has nothing to win there.\n- Any per-op branch, bookkeeping step or extra hash shows up in latency right away (aging reset tracking, extra hashing per miss, the two-pass conservative-update increment).\n- Per-op timings have drifted for unexplained environmental reasons in two cycles running. Small latency deltas between variants can't be trusted unless the variant and its parent are benchmarked side by side.\n\n## Open questions\n\n- Does aging or a doorkeeper help once conservative update limits overestimation? Aging on its own was never tested on the fast-hash base before cycle 2: the cycle-1 attempt failed on infrastructure, and the cycle-2 attempt lacked a fix for accuracy.\n- Can a wider or deeper sketch that keeps row independence add to the accuracy gain at an acceptable metadata cost?\n- Can the conservative-update increment become single-pass or stay in registers, so `get` recovers its latency?\n- How close is workload-a to the hit-rate ceiling for a TinyLFU-family design?\n- Is the unkeyed fast hasher safe against adversarial or collision-prone keys?\n- Was cycle 2's latency regression for compact links real or just noise? This was never confirmed side by side.\n\n## Per-policy notes\n\n- TinyLFU (plain LRU + CountMin admission): this is the current line. The best base is fast hash + compact links + conservative update. The levers that work are estimate accuracy and hot-path cost.\n- W-TinyLFU (window + SLRU): tried once with a ~1% window, it lost on memory and latency with almost no hit-rate gain. Worth revisiting only after the frequency signal improves, and only with low per-hit relink cost.\n- Sampled/frequency-aware eviction: entries near the LRU end have nearly equal estimates, so sampling adds hashing cost for no benefit. Drop it.\n- Aging (Caffeine-style reset): it hurt on its own with a coarse cap and strict ties. It is untested with conservative update or a doorkeeper.\n"
}
```

### 19:58:17 thinking



### 19:58:18 tool: Edit

```json
{
  "replace_all": false,
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "old_string": "Aging on its own was never tested on the fast-hash base before cycle 2: the cycle-1 attempt failed on infrastructure, and the cycle-2 attempt lacked a fix for accuracy.",
  "new_string": "Aging has only been measured once (cycle 2), before conservative update existed. The cycle-1 attempt never ran because of an infrastructure failure."
}
```

### 19:58:28 chat

I've written `cross-cycle-insights.md` to the output directory, rebuilt from the ledger for cycles 0001–0003 using the four required sections in order.

- **What keeps working:** changes that only remove work or shrink the data layout, like fast hashing (cycle 1) and compact `u32` links (cycle 2), improve speed or memory without changing behaviour. The only hit-rate gain so far came from making the frequency estimates more accurate (conservative update, cycle 3). Each cycle's winner became the next base: fast hash, then compact links, then conservative update.
- **What keeps failing:**
  - Adding list structure (window + SLRU, sampling near the LRU end) cost speed and memory with no hit-rate gain.
  - Changing the admission rule or tie handling on its own made hit rate flat or worse.
  - Sketch layouts that break the independence between rows inflated the estimates and hurt admission.
  - Any extra work per operation showed up in latency straight away.
- **Benchmark caution:** latency readings drifted for unexplained environmental reasons in two cycles running. The file says small latency differences need a side-by-side run against the parent before anyone relies on them.
- **Open questions for the next cycles:**
  - Does aging or a doorkeeper help now that conservative update limits overestimation? Aging has only been measured once, in cycle 2, before conservative update existed. The cycle-1 attempt never ran because of a download failure.
  - Can a wider or deeper sketch push accuracy further at an acceptable memory cost?
  - Can the conservative-update increment be done in one pass to win back `get` latency?
  - How close is workload-a to the hit-rate ceiling for this family of designs?
  - Is the fast hasher, which uses no secret key, safe against adversarial keys?
  - Was cycle 2's latency regression real?
- **Per-policy notes:** the file groups observations under plain TinyLFU (the current line), W-TinyLFU, sampled eviction and aging.


---

- finished: 2026-10-20T19:58:29+00:00
- duration: 34.88s
- status: ok
- input_tokens: 6
- output_tokens: 2261
- cache_read_tokens: 57726
- cache_write_tokens: 8423
- cost_usd: 0.12456
