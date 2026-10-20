# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0010\cross-cycle
- allow_edits: True
- started: 2026-07-21T20:14:05+00:00

## timeline

### 20:14:12 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. I need to find the output directory first, then write the file with the four fixed sections.

Let me look for the output directory structure.

### 20:14:12 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 20:14:12 tool: Glob

```json
{
  "pattern": "**/output*/**"
}
```

### 20:14:16 tool: Bash

```json
{
  "command": "ls /",
  "description": "List root directory"
}
```

### 20:14:16 tool: Bash

```json
{
  "command": "ls ~",
  "description": "List home directory"
}
```

### 20:14:20 tool: Glob

```json
{
  "pattern": "**/*.md"
}
```

### 20:14:25 tool: Read

```json
{
  "file_path": "/task.md"
}
```

### 20:14:31 thinking

The allowed roots are:
1. `C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0010\cross-cycle`
2. `C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles-ledger.jsonl`
I need to create the `cross-cycle-insights.md` file in that output directory.

### 20:14:31 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\associative_cache_2026-07-21_16-31-57-327\\cycles\\0010\\cross-cycle"
}
```

### 20:14:34 chat

Now I have the output directory. I'll write the `cross-cycle-insights.md` file as a fresh projection of the 10-cycle ledger.

### 20:15:18 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\associative_cache_2026-07-21_16-31-57-327\\cycles\\0010\\cross-cycle\\cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **3-swap bounded promotion loop.** Extending the promotion budget from one to three swaps resolved the mid-bucket convergence lag on Zipfian workloads. The early-break on a failed swap keeps the hot-key common case at a single iteration, so no measurable latency cost accompanies the hit-rate gain. This has been stable across Cycles 0005–0010 and is now a fixed base component.\n- **32-byte aligned struct.** Padding the compact item struct to 32 bytes eliminated cache-line straddling and halved p99 tail latency. The struct remains below the 40-byte baseline, so it satisfies both the latency and metadata-size objectives simultaneously. Variants that drop this layout invariant consistently reintroduce put-path latency regressions — it must be carried forward in every overlay.\n- **Write-once second-chance eviction flag.** A binary \"ever-accessed\" marker written at first touch (never again on the hot get path) lets the eviction scan skip genuinely warm items and eject cold newcomers. This produced a measurable hit-rate lift over pure LRU tail eviction and is a confirmed base component since Cycle 0006.\n- **LRU-among-cold scan ordering.** Switching the 8-slot second-chance scan to eject the oldest cold candidate (highest index in this positional model) rather than the newest is a free hit-rate improvement at zero latency cost, confirmed in Cycle 0008.\n- **8-slot scan window depth.** This window size is near-optimal: narrower windows increase fallback frequency and spike p99; wider windows add unbounded comparison cost with no hit-rate return. Confirmed stable from Cycles 0006–0010.\n\n## Recurring failures\n\n- **Per-get counter writes (frequency signals on the get path).** Any write of an `lfu_counter` or access-generation field on every cache hit consistently produces tail-latency regressions. The cache-line write cost — not the conditional logic — is the liability. Tested across Cycles 0001–0003 and 0005; closed.\n- **Swap-to-back / LIFO promotion.** Replacing the removal+push pattern with a position-swap heuristic degraded hit rate on Zipfian workloads and recovered no latency. Closed in Cycle 0002.\n- **Two-zone (hot/cold) partitioning.** The hot/cold boundary shifts with every insertion, and at small associativity the hot-zone bypass saves negligible work. Definitively closed in Cycle 0003.\n- **Halving bucket count.** Increased collision density faster than added depth can absorb; hit rate regressed in a pattern consistent with Cycle 0001. Closed in Cycle 0006.\n- **Promotion budget beyond three swaps.** Five-swap and variable-depth teleport variants both regressed (hit rate at five swaps, latency at variable depth). Promotion saturation is reached at three swaps for this Zipfian workload. Closed in Cycles 0006 and 0010.\n- **Second-chance scan and 3-swap loop decoupling.** Reducing swaps to one or zero to isolate the scan's hit-rate benefit eroded hot-item / cold-item separation, making the scan unable to distinguish them. The two mechanisms are coupled and must be kept together. Closed in Cycle 0007.\n- **Compact struct omission.** In three separate cycles (0007, 0009, 0010) variants that overlaid only the eviction policy file silently reverted to the 40-byte layout, reintroducing footprint and latency regressions. This is a process failure as much as a design failure; every variant must explicitly include the `item.rs` overlay.\n- **Flag expiry / generation-counter granularity.** Finer-grained access generations (Cycle 0009 v002) and position-based flag expiry (Cycle 0009 v003) both degraded hit rate and/or latency. The binary ever/never distinction is already the right granularity for this workload.\n- **Conditional scan expansion above the primary window.** The 2-position secondary scan above the 8-slot window fired almost never and produced no measurable effect. Closed in Cycle 0010.\n\n## Open questions\n\n- **Is the Cycle 0006 put p99 regression a live signal or a retired artifact?** Swap-write pressure (falsified in Cycle 0007) and scan direction (falsified in Cycle 0010) have both been ruled out as causes. The most plausible remaining explanation is that it was a transient artifact of the compact-struct transition period, which is now permanently resolved. Continued investigation has diminishing returns.\n- **Insertion-point experiments within strict O(1) bounds.** First-touch teleportation to the warm-cold boundary (Cycle 0010 v001) failed because its swap count scaled with cold-zone depth. A bounded variant — capping first-touch promotion at 5–6 swaps rather than 3 — stays O(1) and has not been tested.\n- **Promotion-budget vs. hit-rate ceiling.** After confirming the 3-swap optimum and ruling out 5-swap, it is unclear whether any promotion change can lift hit rate further without touching the get-path write count. Epoch-based or eviction-time-only frequency updates remain untested.\n- **Measurement floor resolution.** Per-op-time improvements that fall below the ~100 ns timer floor are invisible. Quantifying the floor's effect on hypothesis validity — particularly for latency-reduction claims — would clarify which experiments are actually inconclusive vs. genuinely neutral.\n\n## Per-policy notes\n\n- **Strict LRU tail eviction (baseline).** Provides a clean, O(1) eviction path but does not distinguish cold newcomers from cold survivors. The second-chance flag was introduced to address this gap and has consistently outperformed pure LRU on hit rate since Cycle 0006.\n- **LFU-semantics (frequency counters on get path).** Demonstrated real hit-rate advantages on LIRS/scan-burst traces across Cycles 0001–0005, but every implementation that writes counters on the get path incurred tail-latency regressions. The frequency-ordering benefit at eviction time was also eroded by other workload-specific effects. Read-free or write-deferred LFU has not been tested; per-get LFU writes are closed.\n- **Second-chance / clock-like (write-once flag).** The current confirmed base policy. The binary flag captures the meaningful ever/never split; finer granularity (generations, position-expiry) adds cost without benefit on Zipfian workloads. Scan window is fixed at 8 slots, LRU-among-cold ordering, combined with 3-swap unconditional promotion.\n- **Two-tail frequency comparison at eviction.** Tested in Cycles 0005–0006: overrode clean positional recency, ejecting recently-promoted items before genuine LRU victims. Closed.\n- **Two-zone partitioning.** Closed in Cycle 0003; boundary instability and negligible hot-zone bypass savings make this approach non-viable at small associativity.\n"
}
```

### 20:15:28 chat

`cross-cycle-insights.md` has been written to the output directory. Here's a summary of what was synthesised across the 10 completed cycles:

**Recurring wins** — four confirmed base components that have held across multiple cycles: the 3-swap bounded promotion loop, the 32-byte aligned struct, the write-once second-chance eviction flag, and LRU-among-cold scan ordering (all at an 8-slot fixed window depth).

**Recurring failures** — nine closed avenues: per-get frequency counter writes, swap-to-back promotion, two-zone partitioning, bucket-count halving, promotion beyond three swaps, decoupled scan/promotion, compact struct omission (a process failure recurring in three cycles), flag expiry / generation granularity, and conditional scan expansion.

**Open questions** — four: whether the Cycle 0006 put p99 regression is still live (likely retired), bounded first-touch promotion at 5–6 swaps (untested), whether any promotion change can lift hit rate without get-path writes, and the measurement floor's effect on latency hypothesis validity.

**Per-policy notes** — covers the five distinct eviction policy families tested: baseline LRU tail, per-get LFU, second-chance/clock (current base), two-tail frequency comparison, and two-zone partitioning.


---

- finished: 2026-07-21T20:15:28+00:00
- duration: 83.96s
- status: ok
- input_tokens: 8
- output_tokens: 2873
- cache_read_tokens: 149040
- cache_write_tokens: 12011
- cost_usd: 0.140299
