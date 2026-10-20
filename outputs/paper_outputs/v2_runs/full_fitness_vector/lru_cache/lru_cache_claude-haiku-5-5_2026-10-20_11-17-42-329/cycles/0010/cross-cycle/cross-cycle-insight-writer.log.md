# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T12:08:21+00:00

## timeline

### 12:08:32 thinking



### 12:08:38 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Cheaper hashing on hot paths keeps paying off. Swapping the admission sketch hash and the key-map hasher lowered per-operation cost with no policy change, while extra finish-step mixing added cost and gave nothing back.\n- Node and link narrowing is reliable. Packing flags into spare bits, sentinel indices, and `u32` links shrank nodes without hit-rate loss. Speed has not followed, so treat these as size wins.\n- Admission filtering is the lever that has actually raised hit rate. Gating keys at capacity, first by a sketch and then by a doorkeeper on first sightings, kept one-hit traffic from displacing residents. Moving the hash off the read path made the gate cheaper.\n- Keeping miss-path and demotion work bounded helps tails. A capped second-chance scan and a flag-only retry in demotion both reduced per-operation cost, though each gave up a little hit rate.\n- Consolidating masked index reads into a single helper recovered some cost on the packed layout.\n\n## Recurring failures\n\n- Extra work on the hit path has not paid. Relinks on hit did not move `get`, and per-access hashing added cost to both `get` and `put`.\n- Ghost-driven admission has lost hit rate in both directions (to protected and to probation) on the trace tested, and its per-miss scan adds cost to `put`.\n- Plain CLOCK, and CLOCK with a bounded sweep, lose the protection or reuse window the segmented policy relies on.\n- Placing new keys at the eviction end of probation removes the reuse window and costs hit rate.\n- Shrinking the protected share costs hit rate. Growing it helps hit rate but costs `put` tails.\n- Hoisted or re-structured arena accessors and the single-lookup `put` rewrite did not speed up `get` or `put`.\n- Guarding the reference-bit write cost more than the store it skipped.\n- A mask on every index read adds latency, so packed flags only pay off if the read path stays clean.\n\n## Open questions\n\n- Do the admission gate's hit-rate gains hold on other workloads and capacities, and read-heavy traces in particular? Its aging interval, table size, and table footprint are untested.\n- Does the protected-share effect hold beyond one trace and capacity, and can a larger protected share be had without the `put` tail cost?\n- Can protection be restored without list surgery on the shared path, through a recency signal other than list position?\n- Does the zero-state hasher keep its spread and speed on string keys and non-sequential integers?\n- Are the recurring `get` and `put` tail and median movements real, or a run-to-run artifact of the measurement setup? Re-measurement has not yet settled this.\n- Does the `u32` capacity cap matter for any expected workload?\n\n## Per-policy notes\n\n- Segmented LRU (probation and protected): the protected share is a real hit-rate lever with a `put` cost. Protection depends on the rescued head actually moving, not on a retained flag. A two-bit credit recovered only a small share. The flag-only retry is the cheapest rescue so far.\n- CLOCK: the variants tried (plain and bounded sweep) lost protection or hit rate. Per-slot state in a separate allocation adds cost, so CLOCK has not been a winning base.\n- Ghost ring: no admission target has held hit rate on the trace tested. The idea is weakened and should only be retried with a measured ghost hit rate and a bounded lookup.\n- Admission filter (sketch and doorkeeper): the most promising hit-rate direction. Its cost is hashing on new-key misses at capacity, and read-path work should be kept out of it.\n- Policy-neutral layout and hashing changes (node narrowing, map hasher, index helpers) are the safest wins and apply under any eviction policy.\n"
}
```

### 12:08:42 chat

I wrote `cross-cycle-insights.md` to the cycle 0010 cross-cycle output directory, using the four required sections in order. I built it only from the inlined ledger digest and did not read any previous version of the file.

The main signals across cycles 0001–0010:

- **Recurring wins:** cheaper hashing on hot paths, node and link narrowing (size wins, not speed wins), admission filtering on first sightings (the one lever that has consistently raised hit rate), and bounded miss-path or demotion work.
- **Recurring failures:** extra work on the hit path, ghost-ring admission (lost hit rate in both directions), plain or bounded CLOCK, inserting at the eviction end of probation, and index-accessor restructuring that did not speed up `get` or `put`.
- **Per-policy notes:** the segmented policy's protected share is a trade-off between hit rate and `put` tails; the admission filter is the most promising direction; ghost and CLOCK variants are weakened.

The open questions are mostly about whether the admission and protected-share gains hold beyond workload-a at 2048, and whether the recurring `get` and `put` tail movements are noise.


---

- finished: 2026-10-20T12:08:42+00:00
- duration: 20.58s
- status: ok
- input_tokens: 4
- output_tokens: 3445
- cache_read_tokens: 42417
- cache_write_tokens: 16023
- cost_usd: 0.005352
