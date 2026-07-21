# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0008\cross-cycle
- allow_edits: True
- started: 2026-07-19T23:16:53+00:00

## timeline

### 23:16:59 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. Let me first find the output directory, then write the file.

Let me look for the output directory structure.

### 23:16:59 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 23:16:59 tool: Glob

```json
{
  "pattern": "output/**"
}
```

### 23:17:01 tool: Glob

```json
{
  "pattern": "**/*"
}
```

### 23:17:03 tool: Read

```json
{
  "file_path": "/Users/yaniv/Projects/cache-evolve/task.md"
}
```

### 23:17:06 thinking

Now I have the output directory structure. The working directory is `C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0008\cross-cycle`. I should write the file there.

### 23:17:07 tool: Bash

```json
{
  "command": "ls \"C:/Users/yaniv/Projects/cache-evolve/outputs/s3_fifo_2026-07-20_00-22-12-011/cycles/0008/cross-cycle\"",
  "description": "List output directory contents"
}
```

### 23:17:54 tool: Write

```json
{
  "file_path": "C:/Users/yaniv/Projects/cache-evolve/outputs/s3_fifo_2026-07-20_00-22-12-011/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **O(n) → O(1) algorithmic replacements compound cleanly**: Every substitution of a linear scan with a hash-based O(1) operation yielded large, durable gains — ghost membership via HashSet (cycle 0001), lazy ghost deletion replacing `retain()` (cycle 0002), and full elimination of the `position()` deque scan via `resident_map` (cycle 0005). Each fix was independently additive and survived composition.\n- **Structural consolidation beats decomposition**: Merging value and frequency into a single `resident_map<K,(V,u8)>` (cycle 0005/v003) outperformed every split or parallel-map variant by eliminating redundant hash probes and reducing struct size simultaneously. The single-probe layout is confirmed optimal for this workload.\n- **Small-queue shrinkage was consistently free until it wasn't**: Reducing the small-queue fraction from 1/4 → 1/10 → 1/20 delivered clean, monotone hit-rate and latency gains across cycles 0001–0003. The benefit plateaued at 1/20, and pushing to 1/40 introduced a marginal crossover; 1/80 introduced a clear regression from `evict_small()` call-rate overhead.\n- **Amortising cleanup across future evictions is reliable**: Deferring ghost-deque compaction to capacity-guard sweeps (lazy tombstoning, cycle 0002) eliminated tail spikes without measurable backlog accumulation on Zipfian workloads. Amortisation of bounded cleanup work generalises safely.\n- **Orthogonal improvements compose without interaction penalties**: Changes touching disjoint code paths (queue sizing, ghost-hit path, entry layout, map structure) stacked cleanly in every composite variant tested, and no cross-axis regression was observed when they were combined.\n\n## Recurring failures\n\n- **Bounded/capped scan variants always backfired**: Capping the `evict_main()` second-chance loop to a fixed budget (cycle 0003/v001, cycle 0004) either had no effect (Zipfian cold entries naturally cluster at the tail, so the cap was never binding) or forced eviction of still-hot entries. The uncapped scan is effectively O(1) in practice on Zipfian and should not be re-probed.\n- **Batch-draining evictions amplifies cost, not amortises it**: Concentrating multiple `evict_small()` calls into a single `put()` burst made p50 and p99 worse (cycle 0008/v003). Eviction overhead compounds per-call; one-at-a-time eviction is confirmed optimal.\n- **Widening the promotion gate (freq > 0, freq=1 init, wider second-chance) consistently hurts**: Every attempt to admit entries to main earlier or grant them more dwell time — `freq > 0` ghost promotion (cycle 0003/v003), `freq=1` initialisation on ghost hit (cycle 0007/v002), wider second-chance via higher `MAX_FREQ` (cycle 0007/v003) — either raised `evict_main()` scan depth without recovering hits, or displaced hotter entries. The paper's `freq > 1` gate and `freq=0` ghost initialisation are both load-bearing.\n- **Ghost capacity beyond `main_size` accumulates useless tombstones**: Doubling the ghost window size (cycle 0006/v002) produced no additional recaptured keys because the Zipfian cold tail simply does not re-request within the extended window. `main_size` is the effective tight bound.\n- **Decomposing the unified map restores scan overhead under another name**: Splitting `resident_map` into `data<K,V>` + `freq_map<K,u8>` (cycle 0006/v003) doubled write overhead on every admission and eviction, negating the read-path cache-line benefit. The consolidated layout should not be revisited.\n- **Hash load-factor pre-sizing had zero effect**: Pre-sizing all maps to a ~50% load factor (cycle 0007/v001) produced no measurable latency change, ruling out probe-chain collision depth as a bottleneck at any tested capacity.\n- **Frequency step acceleration increases total eviction work**: Faster frequency drain (step=2, cycle 0004/v003) caused moderately-warm entries to exit main prematurely, raising ghost-hit-driven `evict_main()` invocation rate so that total eviction cost rose even as individual scan lengths shortened. Scan-length reduction and total-eviction-work reduction are decoupled and can oppose each other.\n\n## Open questions\n\n- **What is the dominant per-operation cost on the get path?** All targeted improvements have focused on the put/eviction paths or data-structure layout. The primary fitness signal is anchored to get p50, which has not moved in multiple cycles. The `resident_map` lookup cost under high collision pressure, branch-predictor behavior on the hit-path conditional chain, and memory-access latency for hot-set entries are uncharacterised.\n- **Can `evict_small()` invocation rate be reduced via admission selectivity rather than call amortisation?** Batching is ruled out. A structural approach that reduces the net number of small-queue overflows — e.g., a probabilistic admission gate before inserting into small — might cut `evict_small()` call frequency without burst-cost penalties.\n- **Is the Winners anomaly (net-negative variants passing the threshold) a fitness-function artifact?** Net-negative variants have appeared in the Winners list in at least five consecutive cycles (0003–0008). The fitness function appears to weight a specific latency percentile or aggregate score that these variants happen to clear despite primary-dimension regressions. If the threshold is miscalibrated, it may be generating false-positive continuation signals.\n- **Does the ghost tombstone backlog grow unbounded under dense ghost-hit workloads?** Lazy deletion was validated on Zipfian where tombstones drain steadily, but the backlog behavior under adversarial or uniform workloads has not been characterised.\n\n## Per-policy notes\n\n- **S3-FIFO small queue**: The optimal small-queue fraction on Zipfian is 1/20. Below 1/40 the `evict_small()` call-rate penalty dominates the additional main-queue residency benefit. Above 1/10 the small-queue scan overhead and over-investment in admission filtering dominate. The 1/20 sweet spot is robust and should be held fixed unless the workload distribution changes.\n- **S3-FIFO main queue (`evict_main()`)**: The second-chance loop is effectively O(1) in practice on Zipfian because cold entries cluster at the deque tail; the scan terminates in a few iterations. The deque-churn latency identified in cycle 0003 was eliminated by the `resident_map` redesign (no more freq deque churn). The remaining cost within each call is small and unproductive to target further.\n- **S3-FIFO ghost channel**: The ghost set's role as a dual gate — confirming both frequency (`freq > 1`) and recency (re-request after ghost-listing, not merely while still in small) — is load-bearing and must not be relaxed. `freq=0` on ghost-hit re-admission and the `main_size` ghost capacity bound are both confirmed correct. The ghost `HashSet` membership test on every `put()` call has not been identified as a bottleneck.\n- **`resident_map<K,(V,u8)>` (cycle 0005/v003 baseline)**: This is the confirmed-optimal data structure. Single-probe O(1) for hit detection, value retrieval, and frequency update. Shedding the separate freq deques and Entry value field removed per-entry metadata overhead. Do not decompose, do not add parallel structures.\n"
}
```

### 23:18:02 chat

Written `cross-cycle-insights.md` to the cycle 0008 cross-cycle output directory. The file synthesises all eight cycles into the four fixed sections:

- **Recurring wins**: Five durable patterns — O(n)→O(1) replacements, structural consolidation, small-queue shrinkage up to 1/20, lazy amortisation, and composability of orthogonal changes.
- **Recurring failures**: Eight closed-off avenues — bounded scan caps, batch eviction, relaxed promotion gates, oversized ghost windows, map decomposition, hash pre-sizing, and frequency step acceleration — each with the causal mechanism that ruled it out.
- **Open questions**: Four unresolved threads, prioritised around the get-path bottleneck (primary fitness signal, never directly targeted), admission-selectivity as an alternative to batching, the Winners-list anomaly, and tombstone backlog under non-Zipfian workloads.
- **Per-policy notes**: Configuration values confirmed optimal for S3-FIFO on Zipfian (1/20 small fraction, `freq>1` gate, `freq=0` ghost init, `main_size` ghost cap, unified `resident_map`) with the rationale for each.


---

- finished: 2026-07-19T23:18:03+00:00
- duration: 69.74s
- status: ok
- input_tokens: 7
- output_tokens: 2739
- cache_read_tokens: 122579
- cache_write_tokens: 11185
- cost_usd: 0.126676
