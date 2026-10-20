# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:46:08+00:00

## timeline

### 11:46:20 thinking



### 11:46:27 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Policy-neutral representation changes reliably shrink nodes and structs without costing hit-rate: packing flags into spare bits, dropping per-link discriminants, and narrowing arena links to `u32` with a sentinel (cycles 2, 3, 5).\n- Cheaper hashing on any per-access path lowers cost with no policy change: the admission sketch hash replaced by a cheaper shared hash, the sketch removed from the read path, and a zero-state key-map hasher (cycles 4, 5, 6).\n- Bounding or deferring eviction-time work helps the miss and put tail. A capped second-chance scan and a flag-only retry in the demotion branch both removed unpredictable list work for a small hit-rate give-back (cycles 2, 4).\n- Reuse-aware eviction is the dominant hit-rate direction on the scan-heavy LIRS workload. Segmentation, admission gating, and the ghost ring each gained hit-rate there (cycles 1, 3, 6).\n\n## Recurring failures\n\n- Extra pointer work on shared paths is expensive. Relinking on hit did not move `get` (cycle 2), and list surgery in the protected-side rescue showed up in both `get` and `put` (cycle 3). Moving a head on the hit path to recover protection has not paid off (cycles 4, 5).\n- Rewriting arena index access has repeatedly made `get` or `put` slower, whether through narrowed indices, sentinel links, or hoisted accessors (cycles 2, 3, 4). Narrowing links itself is fine; the access pattern is the problem.\n- Per-access hashing in the admission gate cost both paths until the hash was cheapened (cycle 3). Read-only frequency signals cost hit-rate when removed (cycle 5).\n- Metadata added to recover protection buys little. Extra per-node fields and a two-bit credit each recovered only a small share of the lost protection (cycles 1, 5).\n- Miss-path scans add put cost. The ghost ring's linear lookup over recent evictions added work on every miss (cycle 6).\n- Shrinking the protected segment hurts hit-rate on the scan-heavy trace (cycle 6).\n\n## Open questions\n\n- Do the hit-rate gains from segmentation, admission gating, and the ghost ring hold on workloads without one-hit scans and at other capacities? Every result so far comes from one trace at one size (cycles 1, 3, 4, 6).\n- Is the `get` slowdown after index narrowing and accessor changes caused by codegen or layout? A codegen-only comparison has not been run (cycles 2, 4, 5).\n- Are the `put` and `get` tail movements in the recent Winners real, or run-to-run noise? Several were single runs (cycles 5, 6).\n- Does the zero-state hasher keep its bucket spread on string keys and non-sequential integers, and does the cheap admission hash hold its hit-rate on other workloads (cycles 4, 6)?\n- Can the rescued protected head keep its protection without list surgery on the shared path? Does a different recency approximation close the hit-rate gap (cycles 2, 4, 5)?\n- Can the ghost ring's lookup be bounded with a hashed set, and what is its heap cost? Heap storage is not counted by the struct-size metric (cycle 6).\n- Does the `u32` capacity cap matter for any expected workload (cycle 5)?\n- The cycle 1 variant that was rejected at dependency fetch has never been evaluated. Its strategy remains untested.\n\n## Per-policy notes\n\n- Segmented (probationary and protected) LRU: the hit-rate gain is real on the scan-heavy trace, and the protected share appears to be doing real work, so smaller shares are unlikely to pay off (cycles 1, 6). Rescuing demoted heads recovers some hit-rate but costs pointer work on both `get` and `put`. A flag-only retry is cheaper and gives back a small amount of hit-rate (cycles 3, 4).\n- Second-chance and referenced-bit eviction: a deferred referenced-bit scheme with one bounded rescue, and a capped scan, trade a small hit-rate loss for predictable miss-path cost. Avoiding relinks on overwrite made the put-existing path cheaper (cycles 1, 2).\n- Frequency-sketch admission: the largest hit-rate gain in the cohort, from rejecting one-hit keys at capacity (cycle 3). Its cost has been reduced by cheapening the hash and by feeding the sketch only from writes. Each reduction on the read path has cost a little hit-rate (cycles 4, 5).\n- Ghost ring of recently evicted keys: gains scan resistance by promoting returning keys straight to protected. Its miss-path scan and extra hashing are the cost, so it is worth pursuing only with a bounded lookup (cycle 6).\n"
}
```

### 11:46:30 chat

I wrote `cross-cycle-insights.md` to the cycle 0006 cross-cycle output directory. It has the four required sections in order, built from the ledger digest across cycles 0001–0006. I did not read the previous version of the file.

The main signals:

- **Recurring wins:** Policy-neutral layout changes (narrowed `u32` links, packed flags) shrink nodes without costing hit-rate. Cheaper hashing on hot paths lowers cost. Bounding or deferring eviction-time work helps the miss and put tails. Reuse-aware eviction is the consistent hit-rate direction on the scan-heavy trace.
- **Recurring failures:** Extra pointer work on shared paths has been expensive. Arena index-access rewrites have repeatedly slowed `get` or `put`. Per-node metadata has bought little protection. Shrinking the protected segment hurts hit-rate. Miss-path scans, such as the ghost ring's linear lookup, add put cost.
- **Open questions:** Whether hit-rate gains hold beyond one trace and one capacity. Whether the `get` slowdowns are codegen or layout effects. Whether the recent tail movements are real or noise. Whether the cycle 1 variant that was never evaluated still has value.
- **Per-policy notes:** Grouped by segmented LRU, second-chance/referenced-bit eviction, frequency-sketch admission, and the ghost ring.

The file contains no numbers, following the constraints. Several of the cycle 0006 Winners are labelled as wins in the ledger, but their own insights record trade-offs, so the file describes those trade-offs rather than repeating the Winner labels.


---

- finished: 2026-10-20T11:46:30+00:00
- duration: 22.88s
- status: ok
- input_tokens: 4
- output_tokens: 4688
- cache_read_tokens: 38631
- cache_write_tokens: 13307
- cost_usd: 0.005392
