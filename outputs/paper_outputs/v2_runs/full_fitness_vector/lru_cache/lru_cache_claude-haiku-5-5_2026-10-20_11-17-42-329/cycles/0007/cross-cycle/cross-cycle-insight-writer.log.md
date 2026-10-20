# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:54:21+00:00

## timeline

### 11:54:30 thinking



### 11:54:36 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Policy-neutral representation changes shrink nodes reliably: packing spare bits, narrowing arena links, and sentinel-indexed links all cut size without costing hit-rate. They have not moved `get` much, so credit them for size, not speed.\n- Cheap hashing on the key-map and admission paths lowers per-operation cost: a cheaper shared hash for the sketch, and a zero-state map hasher, both helped without changing policy.\n- Removing sketch hashing from the read path is a clean read-cost reduction, with only a small hit-rate price on the one trace tested.\n- Avoiding list surgery on shared paths tends to pay: the flag-only demotion retry won on per-operation cost, and deferring recency work to eviction time helped `put` more than `get`.\n- A bounded second-chance scan gives a predictable miss-path tail, the most consistent per-operation gain in the cohort.\n- Reuse-aware and admission-gated eviction is the direction that consistently raises hit-rate on the LIRS workload-a trace.\n\n## Recurring failures\n\n- Per-access work on the hit path rarely pays. Admission hashing on every access, pointer work in the protected rescue, and extra per-lookup mixing all raised `get`/`put` cost without a matching hit-rate gain.\n- Ghost-style recently-evicted tracking has weakened. It gained scan resistance, but its miss-path scan and hashing added cost, and promoting ghost hits straight to protected lost hit-rate.\n- Hoisted or masked arena index accessors slowed `get` and `put` in more than one cycle. Changes to the shared index helpers are the most reliable source of regressions.\n- Growing per-node or per-cache metadata without a clear hit-rate gain has not paid off.\n- Shrinking the protected segment cost hit-rate on the scan-heavy trace.\n- Removing the list rotation on hit weakened protection. A retained flag or credit recovered only a small part of it; the head has to actually move.\n\n## Open questions\n\n- Do the hit-rate gains of admission gating, ghost tracking, and protected-share tuning hold on workloads without one-hit scans, at other capacities, and on read-heavy traces?\n- Is the `get` slowdown after index-width or accessor changes caused by codegen or layout? A codegen-only comparison has not yet been run.\n- Are the `put` tail movements and single-run `get` deltas real, or run-to-run noise? Most have not been re-measured.\n- Does the zero-state hasher keep its spread and speed on string keys and non-sequential integer keys?\n- Can protection come from a recency signal other than list position, without list surgery on the shared path?\n- Several Winner labels rest on size alone or on unverified attributions. Do the fitness vectors support a per-operation gain for each?\n\n## Per-policy notes\n\n- Segmented (probationary/protected): the protected share appears to do real work; shrinking it hurt hit-rate. Delayed-demotion rescues recover some hit-rate but add pointer work. A flag-only retry is the cheaper variant, but it gives back some protection.\n- Second-chance / referenced-bit: bounded scans give a predictable miss tail. Deferring recency work helps `put` more than `get`. A two-bit credit recovers only a small share of lost protection.\n- Admission sketch (frequency-filtered): the largest hit-rate gain seen, from rejecting one-hit keys at capacity. Its cost is per-access hashing, which is now cheaper but still an overhead. Read-path removal lowers cost with a small hit-rate loss; it has not been tested on read-heavy traces.\n- Ghost / recently-evicted ring: scan resistance, but at a miss-path cost and with heap storage that struct-size metrics miss. Keep ghost hits probationary if this is retried, and measure the ghost's own hit rate before changing admission.\n"
}
```

### 11:54:39 chat

I wrote `cross-cycle-insights.md` to the cross-cycle output directory for cycle 0007. It has the four required sections in order, built only from the ledger digest in the prompt. I did not read the previous file, and I did not invent any numbers.

- **Recurring wins:** policy-neutral layout changes that shrink nodes, cheaper hashing on the key-map and admission paths, avoiding list surgery on shared paths, the bounded second-chance scan, and admission-gated eviction for hit-rate on workload-a.
- **Recurring failures:** per-access work on the hit path, ghost-style tracking (its miss-path cost and the loss from promoting ghost hits to protected), hoisted or masked index accessors, metadata growth without hit-rate gains, shrinking the protected segment, and removing the hit-path rotation.
- **Open questions:** whether the admission and ghost gains generalise beyond workload-a at 2048, whether the `get` slowdowns are codegen effects, whether `put` tail movements are noise, hasher behaviour on string keys, and whether the Winner labels are backed by the fitness vectors.
- **Per-policy notes:** grouped by segmented, second-chance, admission sketch, and ghost ring.

I did not commit anything.


---

- finished: 2026-10-20T11:54:39+00:00
- duration: 18.03s
- status: ok
- input_tokens: 4
- output_tokens: 3284
- cache_read_tokens: 39531
- cache_write_tokens: 12940
- cost_usd: 0.004626
