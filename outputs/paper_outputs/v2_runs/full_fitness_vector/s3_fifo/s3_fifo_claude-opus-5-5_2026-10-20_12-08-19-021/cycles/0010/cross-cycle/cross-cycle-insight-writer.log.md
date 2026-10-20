# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:53:44+00:00

## timeline

### 14:54:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0010)\n\n## Recurring wins\n\n- **The adaptive small/main split holds its margin.** It has survived every replacement tried: ARC-direction signals (0003), the static 10/90 split (0003, 0004) and frozen splits with the incumbent graduation rule kept (0004). The adaptation itself earns hit-rate, not just its starting ratio.\n- **Hit-rate moves only through policy bounds and retention depth, and only in small steps.** The lowered small floor (`capacity/64`, 0005, reproduced on another layout in 0006) and deeper linear `freq` retention (`MAX_FREQ` 7, 0008; bounded in 0010) are the only levers that have raised hit-rate.\n- **Bounding drain work separates retention depth from `put` cost.** Linear decrement with a per-call rotation budget (0010) recovers most of the depth gain without the `put` tail. Halving decay (0009) did not.\n- **The `get` path should touch only memory it already reads.** Removing O(capacity) scans (0002), splitting cold fields such as the key out of the hot array (0006) and bit-packing `freq` into the probed index bucket (0008) all helped `get` or metadata with no trade-off.\n- **`put` gains come from cutting real machine work.** The wins came from removing clones, compaction, modulo wraps and resize branches (0003, 0004). Count the parent's actual work before claiming a lever.\n- **Policy and layout levers stack freely across lineages.** Policy effects reproduce exactly on different storage layouts (0006).\n\n## Recurring failures\n\n- **Every change to the ghost path has lost hit-rate, in both directions.** Stronger readmission failed: ghosting main victims (0002), readmit with extra `freq` (0005) and a full-capacity ghost (0008). Weaker readmission failed too: routing ghost hits through small (0009). Direct ghost-to-main readmission is what links the split shrink to filling main, and it looks like a local optimum.\n- **Trigger-rate tuning is inert.** Damping grow-small (0006) and doubling the shrink step (0007) did not move hit-rate.\n- **The upper clamp never binds.** On workload-a small only shrinks from its starting quarter, so lowering the ceiling (0010) did nothing. Lowering the floor further is also discouraged.\n- **Graduation-rule and graduate-credit changes are neutral or negative.** This covers the looser threshold (0001/0002), `freq >= 1` (0003) and resetting freq on graduation (0010).\n- **Layouts that widen the probe or add a dependent access slow `get`.** This covers the inline payload in hash buckets (0005, 0006), a separate `freqs` array (0007) and a custom linear-probe table, which also moved cost onto `put` (0009).\n- **Changing two levers at once makes losses impossible to attribute** (0003). \"Single-lever\" split changes have repeatedly coupled the ghost size to main size (0004).\n- **`get` micro-trims such as store elision are below benchmark resolution** (0007).\n\n## Open questions\n\n- **Rotation-budget size.** How often does the budget force releases (`forced_evictions`), and would a larger budget close the rest of the gap to the unbounded `MAX_FREQ` 7 hit-rate without bringing back the `put` tail?\n- **Where `small_size` settles.** Does `settled_small` sit at the floor, and how often (`floor_blocks`)? This has been unreported for ten cycles because the split-report stdout never reaches the insight writers.\n- **Which latency deltas are real.** `get` moves regularly when the `get` code is byte-identical, and drift went in both directions within single cohorts (0007, 0010). This needs same-run parent-vs-child benchmarking.\n- **The `get` floor.** Can it drop below hash + bucket + `slots[i].value`, for example with a prefetch from the probed bucket, without widening the index?\n- **The real memory footprint.** `metadata-size` counts only `Entry`, so it misses ghost heap and parallel arrays such as `keys`. Is a niche-free `keys` array a true per-slot saving?\n- **Other workloads.** Does small ever grow above its initial quarter on any workload (Zipf sweeps, `max_small`, `ceil_blocks`)?\n\n## Per-policy notes\n\n- **S3-FIFO (adaptive split, the only lineage).** The current parent is 0010 v001:\n  - hashbrown index with packed bits, carrying a 3-bit `freq` (`MAX_FREQ` = 7) and the slot id;\n  - linear decrement, with a rotation budget per `evict_main`;\n  - floor at `capacity/64`, ceiling at `capacity/2`;\n  - direct ghost-to-main readmission, fingerprint-ring ghost;\n  - hot and cold fields kept in separate arrays (key stored separately), 8-byte `Entry`.\n- **S3-FIFO ghost.** The fingerprint ring's shorter effective memory nudged the split in a favourable direction (0003). Ghost size and contents act as a policy knob that feeds the adaptation, so pin ghost size whenever you test split levers.\n- **S3-FIFO small/main sizing.** The ghost-hit shrink side sets the balance point. The floor matters weakly, while the ceiling and both trigger rates are inert on workload-a.\n- **S3-FIFO main CLOCK.** Retention laps drive hit-rate, and drain cost lands on `put`. In mixed streams it also shows up in `get`.\n- **Static S3-FIFO (10/90, paper rules).** It loses a little hit-rate to the adaptive split and isn't worth revisiting unless ghost size is pinned.\n"
}
```

### 14:54:13 chat

I've written `cross-cycle-insights.md` to the output dir with the four required sections, built from cycles 0001–0010 of the ledger. No numbers were invented.

- **Recurring wins:** The adaptive small/main split has beaten every replacement tried. Hit-rate only moves through policy bounds (the `capacity/64` floor) and retention depth (`MAX_FREQ` 7). Bounding the eviction work keeps most of that depth gain without the `put` slowdown. `get` improves when it only reads memory it already touches, such as `freq` packed into the index bucket. `put` improves when real work is removed.
- **Recurring failures:** Every change to the ghost path has lost hit-rate, whether readmission was made stronger or weaker. Tuning how fast the split moves does nothing, and the upper clamp never binds. Changing graduation rules didn't help. Layouts that widen the probe or add a second memory access slow `get`. Several variants changed two levers at once, so their losses can't be attributed.
- **Open questions:**
  - Where should the rotation budget be set? This is the next step for the current parent.
  - Where does `small_size` actually settle? Ten cycles have asked; the variants generate the split-report output, but it never reaches the insight writers.
  - Which latency changes are real? That needs parent and child benchmarked in the same run.
  - Can `get` get faster than it is now?
  - What is the real memory footprint? `metadata-size` misses ghost and parallel-array memory.
  - How does the split behave on other workloads?
- **Per-policy notes:** These cover the current S3-FIFO parent (0010 v001), with separate notes on the ghost, split sizing, the main CLOCK queue, and why the static 10/90 split was dropped.


---

- finished: 2026-10-20T14:54:14+00:00
- duration: 30.18s
- status: ok
- input_tokens: 4
- output_tokens: 2665
- cache_read_tokens: 44168
- cache_write_tokens: 17581
- cost_usd: 0.202798
