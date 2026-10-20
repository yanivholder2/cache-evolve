# Cross-Cycle Insights (through cycle 0010)

## Recurring wins

- **The adaptive small/main split holds its margin.** It has survived every replacement tried: ARC-direction signals (0003), the static 10/90 split (0003, 0004) and frozen splits with the incumbent graduation rule kept (0004). The adaptation itself earns hit-rate, not just its starting ratio.
- **Hit-rate moves only through policy bounds and retention depth, and only in small steps.** The lowered small floor (`capacity/64`, 0005, reproduced on another layout in 0006) and deeper linear `freq` retention (`MAX_FREQ` 7, 0008; bounded in 0010) are the only levers that have raised hit-rate.
- **Bounding drain work separates retention depth from `put` cost.** Linear decrement with a per-call rotation budget (0010) recovers most of the depth gain without the `put` tail. Halving decay (0009) did not.
- **The `get` path should touch only memory it already reads.** Removing O(capacity) scans (0002), splitting cold fields such as the key out of the hot array (0006) and bit-packing `freq` into the probed index bucket (0008) all helped `get` or metadata with no trade-off.
- **`put` gains come from cutting real machine work.** The wins came from removing clones, compaction, modulo wraps and resize branches (0003, 0004). Count the parent's actual work before claiming a lever.
- **Policy and layout levers stack freely across lineages.** Policy effects reproduce exactly on different storage layouts (0006).

## Recurring failures

- **Every change to the ghost path has lost hit-rate, in both directions.** Stronger readmission failed: ghosting main victims (0002), readmit with extra `freq` (0005) and a full-capacity ghost (0008). Weaker readmission failed too: routing ghost hits through small (0009). Direct ghost-to-main readmission is what links the split shrink to filling main, and it looks like a local optimum.
- **Trigger-rate tuning is inert.** Damping grow-small (0006) and doubling the shrink step (0007) did not move hit-rate.
- **The upper clamp never binds.** On workload-a small only shrinks from its starting quarter, so lowering the ceiling (0010) did nothing. Lowering the floor further is also discouraged.
- **Graduation-rule and graduate-credit changes are neutral or negative.** This covers the looser threshold (0001/0002), `freq >= 1` (0003) and resetting freq on graduation (0010).
- **Layouts that widen the probe or add a dependent access slow `get`.** This covers the inline payload in hash buckets (0005, 0006), a separate `freqs` array (0007) and a custom linear-probe table, which also moved cost onto `put` (0009).
- **Changing two levers at once makes losses impossible to attribute** (0003). "Single-lever" split changes have repeatedly coupled the ghost size to main size (0004).
- **`get` micro-trims such as store elision are below benchmark resolution** (0007).

## Open questions

- **Rotation-budget size.** How often does the budget force releases (`forced_evictions`), and would a larger budget close the rest of the gap to the unbounded `MAX_FREQ` 7 hit-rate without bringing back the `put` tail?
- **Where `small_size` settles.** Does `settled_small` sit at the floor, and how often (`floor_blocks`)? This has been unreported for ten cycles because the split-report stdout never reaches the insight writers.
- **Which latency deltas are real.** `get` moves regularly when the `get` code is byte-identical, and drift went in both directions within single cohorts (0007, 0010). This needs same-run parent-vs-child benchmarking.
- **The `get` floor.** Can it drop below hash + bucket + `slots[i].value`, for example with a prefetch from the probed bucket, without widening the index?
- **The real memory footprint.** `metadata-size` counts only `Entry`, so it misses ghost heap and parallel arrays such as `keys`. Is a niche-free `keys` array a true per-slot saving?
- **Other workloads.** Does small ever grow above its initial quarter on any workload (Zipf sweeps, `max_small`, `ceil_blocks`)?

## Per-policy notes

- **S3-FIFO (adaptive split, the only lineage).** The current parent is 0010 v001:
  - hashbrown index with packed bits, carrying a 3-bit `freq` (`MAX_FREQ` = 7) and the slot id;
  - linear decrement, with a rotation budget per `evict_main`;
  - floor at `capacity/64`, ceiling at `capacity/2`;
  - direct ghost-to-main readmission, fingerprint-ring ghost;
  - hot and cold fields kept in separate arrays (key stored separately), 8-byte `Entry`.
- **S3-FIFO ghost.** The fingerprint ring's shorter effective memory nudged the split in a favourable direction (0003). Ghost size and contents act as a policy knob that feeds the adaptation, so pin ghost size whenever you test split levers.
- **S3-FIFO small/main sizing.** The ghost-hit shrink side sets the balance point. The floor matters weakly, while the ceiling and both trigger rates are inert on workload-a.
- **S3-FIFO main CLOCK.** Retention laps drive hit-rate, and drain cost lands on `put`. In mixed streams it also shows up in `get`.
- **Static S3-FIFO (10/90, paper rules).** It loses a little hit-rate to the adaptive split and isn't worth revisiting unless ghost size is pinned.
