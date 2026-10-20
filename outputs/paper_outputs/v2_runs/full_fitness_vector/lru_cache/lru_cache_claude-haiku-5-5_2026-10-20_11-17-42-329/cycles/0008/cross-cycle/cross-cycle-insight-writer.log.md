# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:58:57+00:00

## timeline

### 11:59:08 thinking



### 11:59:14 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Policy-neutral layout changes that shrink nodes or structs have held up: narrowed arena links with a sentinel index, and flags packed into spare link bits with the masking consolidated into one helper. Hit-rate stayed intact in each case.\n- Cheaper hashing on hot paths lowers per-operation cost without changing policy: the zero-state key-map hasher and the shared admission-sketch hash both helped. Extra finish-step mixing added cost and gave nothing.\n- Deferring recency work to eviction time, and keeping a rescued protected head in place rather than moving it, cut put-path cost.\n- Bounding the second-chance eviction scan trades a small hit-rate loss for a predictable miss-path tail.\n- Consolidating masked index reads into one helper recovered per-operation cost on the packed layout.\n\n## Recurring failures\n\n- Work added to every access on shared paths keeps costing more than it returns: per-access sketch hashing, finish-step mixing, and the flag mask on every link read.\n- List surgery on hit or overwrite paths for recency protection has not paid off. Removing relinks did not move `get`, and restoring protection by moving the head cost hit-rate or per-operation time.\n- Ghost-ring admission has lost hit-rate in both directions (promotion to protected, and promotion to probation) on workload-a at 2048.\n- Shrinking the protected segment hurt hit-rate on the scan-heavy trace.\n- A credit bit in the flags byte recovers only a small part of the protection lost when list rotation is dropped.\n- Rewriting the `put` miss path to a single key-map lookup did not reduce `put` cost.\n- Index-access rewrites (narrowed indices, hoisted accessors) slowed `get` or `put` without a confirmed cause.\n\n## Open questions\n\n- Do the hit-rate gains from admission gating and the segmented policy hold on other workloads and capacities? Most results come from workload-a at 2048.\n- Are the recurring `get` and `put` tail movements real or run-to-run noise? Re-measure before crediting or blaming any mechanism.\n- Does the zero-state hasher keep its speedup and bucket spread on string keys and non-sequential integer keys?\n- Is the index-width slowdown caused by codegen or accessor structure? A codegen-only comparison has not been run.\n- Can protection for rescued heads be restored without list surgery on the shared path?\n- Several insights were written from hypothesis text and fitness vectors because diffs could not be inspected, and the v002 node growth in cycle 7 is unexplained. Code-level attributions remain unverified.\n- The cycle 1 v003 strategy (reference-aware eviction) was never evaluated because of a dependency fetch failure.\n\n## Per-policy notes\n\n- Segmented (probationary/protected) LRU: the 4/5 protected share appears to be doing real work. Protected-side rescue recovers hit-rate at a per-operation cost. A flag-only retry is cheaper but gives back a little protection.\n- Second-chance / referenced-bit: deferred recency and a bounded scan give the clearest put-tail gains. Hit-rate on re-referenced keys drops slightly.\n- Frequency-sketch admission: gains hit-rate by rejecting one-hit keys at capacity, but its hashing has been the main cost. Removing the read-path hash lowers read cost and costs a small hit-rate loss, since read-only popularity no longer counts.\n- Ghost ring: no promotion target has held hit-rate on workload-a at 2048. Its per-miss scan and extra hashing land in `put`, and its heap storage is not in the struct-size metric.\n"
}
```

### 11:59:16 chat

I wrote `cross-cycle-insights.md` to the cycle 0008 cross-cycle output directory. It has the four required sections in order, built only from the ledger digest.

- **Recurring wins:** policy-neutral node shrinking (sentinel links, packed flags), cheaper hashing on hot paths, deferring recency work to eviction time, bounded second-chance scans, and consolidated index access.
- **Recurring failures:** per-access work on shared paths, list surgery on hit or overwrite paths, ghost-ring admission (lost hit-rate both ways on workload-a at 2048), shrinking the protected segment, and index-width rewrites that slowed get or put without a confirmed cause.
- **Open questions:** whether gains hold on other workloads and capacities, whether recurring tail movements are noise, hasher behaviour on string and non-sequential keys, and the unevaluated cycle 1 v003 strategy.
- **Per-policy notes:** segmented LRU, second-chance, frequency-sketch admission, and the ghost ring.

I did not read the previous version of the file. The write succeeded without an overwrite error, so no prior file was present.


---

- finished: 2026-10-20T11:59:17+00:00
- duration: 19.66s
- status: ok
- input_tokens: 4
- output_tokens: 3449
- cache_read_tokens: 40397
- cache_write_tokens: 14008
- cost_usd: 0.00493
