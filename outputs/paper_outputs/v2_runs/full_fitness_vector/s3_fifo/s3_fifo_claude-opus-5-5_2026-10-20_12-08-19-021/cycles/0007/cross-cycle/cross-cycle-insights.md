# Cross-Cycle Insights (through cycle 0007)

## Recurring wins

- **The adaptive small/main split is the durable policy core.** It came in at 0001 and has beaten every challenger since: ARC-direction signals, the published static 10/90 split, a frozen 10/90 split with the incumbent graduation rule kept, and ghost resizing. It is a package (adaptation plus `freq > 1` graduation plus clamps), and it absorbs most perturbations to other levers.
- **The adaptation's bounds are the only policy lever that has moved hit-rate since 0001.** Lowering the small-queue floor to `capacity/64` gave a small gain (0005). The gain reproduced exactly on a different storage layout (0006).
- **Removing O(n) and machine work gives real latency wins.** These include hashed indexing instead of linear scans (0002), the fingerprint ring ghost with no clones or compaction (0003), and replacing modulo with a compare plus inlined eviction on `put` (0004). Every credible latency gain traces to a specific removed piece of work on the measured path.
- **Splitting hot and cold data with a narrow index helps `get`.** The key moved out to a parallel array behind the `(K, u32)` index (0006). The `get` path should touch only the fields it reads together (`value`, `freq`).
- **Policy and layout levers are independent and stack freely across lineages.** Policy-neutral refactors stayed policy-exact, and policy gains carried over unchanged across layouts.

## Recurring failures

- **Strengthening the movement or ghost path backfires.** This held for ghosting main victims (0002), extra ghost traffic or ARC two-ghost signals (0003), and readmit-with-`freq` credit (0005). Extra ghost hits push the split around or keep one-time-reuse keys in main for too long.
- **Neither trigger rate is a lever.** Damping grow-small (0006) and doubling the ghost-hit shrink step (0007) were both inert on hit-rate. Tuning how fast the split moves does nothing; only where its bounds sit matters.
- **Static splits lose,** both the wholesale replacement (0003) and the one-lever version (0004). Ghost-size tuning lost too: shrinking the ghost was absorbed by the clamps.
- **Two-lever and coupled variants can't be read.** Variants that changed the split together with graduation or ghost size (0003, 0004) could not attribute their losses. Ghost size kept getting tied to main size by accident.
- **Wide or inline probe-table layouts slow `get`.** Inline payloads gave buckets whose size is not a power of two (0005 and 0006). Splitting fields that every hit reads together into separate arrays (`freqs`, 0007) costs a second access for little locality gain at cache-resident capacity.
- **Micro-trims below the benchmark's noise floor** (store elision on `get`, 0007) can't be resolved by this benchmark.
- **Rank-only Winners.** In most cycles at least one "Winner" lost to its parent on both axes and got in only on cohort rank. Don't treat Winner status as evidence a lever works.

## Open questions

- **Where does `small_size` actually settle, and does the floor bind (`floor_blocks`)?** Seven cycles have asked. The split reports are generated but never reach the insight writers. The loop must copy that stdout into the eval artifacts.
- **Is ghost capacity, held relative to the initial main size, now what caps hit-rate?** It was only ever tested together with other levers or absorbed by the clamps. Test it with the clamps fixed.
- **Is a static split at the settled value as good as adaptation,** with ghost size pinned at the parent's value? This is still the clean test that separates adaptation from ratio.
- **Can `freq` be bit-packed next to the value or index entry?** That would keep the 8-byte `Entry` without a second access per hit.
- **Would a niche-free `keys: Vec<K>` with liveness tracked through `free` really shrink the per-slot footprint?** `metadata-size` alone overstates this saving.
- **Conditional readmit credit** (only on a second ghost hit, or only while small is at its floor) is untested.
- **Several latency deltas are still unverified:** the `get` cost of `freqs`, the `put` cost of a longer main, and others. These need same-run parent-versus-child benchmarking.

## Per-policy notes

- **S3-FIFO (adaptive split lineage):** current parent lineage is 0006 v002, with the `capacity/64` floor, a narrow `(K, u32)` index, the hot/cold split with keys out of the slab, and the fingerprint ring ghost. Carry 0007 v001 (`freqs` split) only as a metadata-size branch.
- **S3-FIFO ghost:** it acts as a policy knob, not plumbing. Its effective memory, including hole handling in the ring, feeds the split adaptation directly. Pin its size whenever another lever is under test.
- **Metrics caveats:** `metadata-size` counts only `size_of::<Entry>`. It hides all ghost heap and parallel-array costs, so treat it as a proxy for the hot array. `get` latency often moves when the `get` code is byte-identical to the parent's, and p99 is especially noisy. Baseline drift (0007) can shift every variant at once. Credit latency only to changes on the path that was measured, and only with a same-run re-baseline.
- **Infrastructure:** build failures (Maven 429s in 0001) can reject variants for reasons unrelated to their code. Re-submit those variants rather than counting them as negative results.
